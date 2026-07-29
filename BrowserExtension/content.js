// TextWarden Web — content script.
// Watches the focused <textarea> / contenteditable, checks its text against the
// local LanguageTool server (via the background worker), draws underline overlays
// (contenteditable only), and offers one-click fixes from a suggestion panel.
//
// Offsets: LanguageTool returns UTF-16 code-unit offsets — JavaScript string
// indices are UTF-16 code units, so no conversion is needed here.

(() => {
  "use strict";

  const api = globalThis.browser ?? globalThis.chrome;

  const DEBOUNCE_MS = 600;
  const MAX_TEXT_LENGTH = 20000;
  const MAX_SUGGESTIONS = 3;

  let activeField = null;
  let debounceTimer = null;
  let lastCheckedText = null;
  let matches = [];
  let serverOffline = false;

  // UI elements (created lazily, one set per frame)
  let badge = null;
  let panel = null;
  let underlineLayer = null;

  // ---------------------------------------------------------------- field discovery

  function editableRoot(node) {
    if (!node || node.nodeType !== Node.ELEMENT_NODE) return null;
    if (node.tagName === "TEXTAREA") {
      return node.readOnly || node.disabled ? null : node;
    }
    if (node.isContentEditable) {
      let root = node;
      while (root.parentElement && root.parentElement.isContentEditable) {
        root = root.parentElement;
      }
      return root;
    }
    return null;
  }

  function isTextarea(field) {
    return field.tagName === "TEXTAREA";
  }

  // ---------------------------------------------------------------- text extraction
  //
  // For contenteditable we walk text nodes, recording each node's span in the
  // combined string, and insert "\n" between different block ancestors so
  // LanguageTool sees sentence boundaries (Gmail wraps each line in a <div>).

  const BLOCK_TAGS = new Set([
    "DIV", "P", "LI", "BLOCKQUOTE", "H1", "H2", "H3", "H4", "H5", "H6",
    "PRE", "TD", "TH", "TR", "UL", "OL", "BR",
  ]);

  function nearestBlock(node, root) {
    let el = node.parentElement;
    while (el && el !== root) {
      if (BLOCK_TAGS.has(el.tagName)) return el;
      el = el.parentElement;
    }
    return root;
  }

  function extractRichText(root) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    const nodeMap = []; // { node, start, end } in combined-string offsets
    let text = "";
    let previousBlock = null;

    let node = walker.nextNode();
    while (node) {
      const block = nearestBlock(node, root);
      if (previousBlock && block !== previousBlock && text.length && !text.endsWith("\n")) {
        text += "\n";
      }
      previousBlock = block;

      const start = text.length;
      text += node.nodeValue;
      nodeMap.push({ node, start, end: text.length });
      node = walker.nextNode();
    }
    return { text, nodeMap };
  }

  function fieldText(field) {
    if (isTextarea(field)) {
      return { text: field.value, nodeMap: null };
    }
    return extractRichText(field);
  }

  // Resolve a combined-string offset to a (textNode, offsetInNode) position
  function resolvePosition(nodeMap, offset) {
    for (const entry of nodeMap) {
      if (offset >= entry.start && offset <= entry.end) {
        return { node: entry.node, offset: offset - entry.start };
      }
    }
    return null;
  }

  // ---------------------------------------------------------------- checking

  function scheduleCheck() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(runCheck, DEBOUNCE_MS);
  }

  function runCheck() {
    if (!activeField) return;
    const { text } = fieldText(activeField);

    if (text.trim().length < 2 || text.length > MAX_TEXT_LENGTH) {
      matches = [];
      render();
      return;
    }
    if (text === lastCheckedText) return;
    lastCheckedText = text;

    api.runtime.sendMessage({ type: "lt-check", text }, (response) => {
      // Field changed or unfocused while the request was in flight → drop
      if (!activeField) return;
      const { text: currentText } = fieldText(activeField);
      if (currentText !== text) return;

      if (!response || response.error) {
        serverOffline = true;
        matches = [];
      } else {
        serverOffline = false;
        matches = (response.matches || []).filter((m) => m.length > 0);
      }
      render();
    });
  }

  // ---------------------------------------------------------------- applying fixes

  function applyFix(match, replacement) {
    if (!activeField) return;
    const field = activeField;

    if (isTextarea(field)) {
      const value = field.value;
      field.value =
        value.slice(0, match.offset) + replacement + value.slice(match.offset + match.length);
      field.dispatchEvent(new Event("input", { bubbles: true }));
    } else {
      const { text, nodeMap } = extractRichText(field);
      // Guard: only apply if the flagged text is still where LT said it was
      if (text.substr(match.offset, match.length) !== textAt(match)) return;
      const start = resolvePosition(nodeMap, match.offset);
      const end = resolvePosition(nodeMap, match.offset + match.length);
      if (!start || !end) return;

      const range = document.createRange();
      range.setStart(start.node, start.offset);
      range.setEnd(end.node, end.offset);
      range.deleteContents();
      range.insertNode(document.createTextNode(replacement));
      field.normalize();
      field.dispatchEvent(new Event("input", { bubbles: true }));
    }

    lastCheckedText = null;
    hidePanel();
    runCheck();
  }

  function textAt(match) {
    const context = match.context;
    if (!context) return null;
    return context.text.substr(context.offset, context.length);
  }

  // ---------------------------------------------------------------- UI

  function ensureUI() {
    if (badge) return;

    badge = document.createElement("div");
    badge.className = "textwarden-badge";
    badge.addEventListener("mousedown", (event) => {
      event.preventDefault(); // keep focus in the field
      event.stopPropagation();
      togglePanel();
    });

    panel = document.createElement("div");
    panel.className = "textwarden-panel";
    panel.addEventListener("mousedown", (event) => {
      event.preventDefault();
      event.stopPropagation();
    });

    underlineLayer = document.createElement("div");
    underlineLayer.className = "textwarden-underlines";

    document.documentElement.append(underlineLayer, badge, panel);
  }

  function render() {
    if (!activeField) return;
    ensureUI();
    positionBadge();
    renderBadge();
    renderUnderlines();
    if (panel.style.display === "block") renderPanel();
  }

  function renderBadge() {
    badge.style.display = "block";
    if (serverOffline) {
      badge.textContent = "–";
      badge.dataset.state = "offline";
      badge.title = "TextWarden: grammar server is off (use the Project Console to start it)";
    } else if (matches.length === 0) {
      badge.textContent = "✓";
      badge.dataset.state = "clean";
      badge.title = "TextWarden: no issues found";
    } else {
      badge.textContent = String(matches.length);
      badge.dataset.state = "issues";
      badge.title = `TextWarden: ${matches.length} issue(s) — click to review`;
    }
  }

  function positionBadge() {
    const rect = activeField.getBoundingClientRect();
    badge.style.top = `${window.scrollY + rect.bottom - 22}px`;
    badge.style.left = `${window.scrollX + rect.right - 22}px`;
  }

  function renderUnderlines() {
    underlineLayer.textContent = "";
    if (isTextarea(activeField) || serverOffline) return; // textarea text isn't in the DOM

    const { text, nodeMap } = extractRichText(activeField);
    if (text !== lastCheckedText) return;

    for (const match of matches) {
      const start = resolvePosition(nodeMap, match.offset);
      const end = resolvePosition(nodeMap, match.offset + match.length);
      if (!start || !end) continue;

      const range = document.createRange();
      try {
        range.setStart(start.node, start.offset);
        range.setEnd(end.node, end.offset);
      } catch {
        continue;
      }
      for (const rect of range.getClientRects()) {
        const mark = document.createElement("div");
        mark.className = "textwarden-mark";
        mark.style.top = `${window.scrollY + rect.bottom - 2}px`;
        mark.style.left = `${window.scrollX + rect.left}px`;
        mark.style.width = `${rect.width}px`;
        underlineLayer.appendChild(mark);
      }
    }
  }

  function togglePanel() {
    if (panel.style.display === "block") {
      hidePanel();
    } else {
      renderPanel();
      panel.style.display = "block";
      positionPanel();
    }
  }

  function hidePanel() {
    if (panel) panel.style.display = "none";
  }

  function positionPanel() {
    const rect = activeField.getBoundingClientRect();
    const top = window.scrollY + rect.bottom + 6;
    const left = Math.max(8, window.scrollX + rect.right - 320);
    panel.style.top = `${top}px`;
    panel.style.left = `${left}px`;
  }

  function renderPanel() {
    panel.textContent = "";

    const header = document.createElement("div");
    header.className = "textwarden-panel-header";
    header.textContent = serverOffline
      ? "Grammar server is off"
      : matches.length
        ? `${matches.length} suggestion(s)`
        : "No issues found";
    panel.appendChild(header);

    for (const match of matches) {
      const item = document.createElement("div");
      item.className = "textwarden-item";

      const flagged = textAt(match);
      if (flagged) {
        const quoted = document.createElement("span");
        quoted.className = "textwarden-flagged";
        quoted.textContent = flagged;
        item.appendChild(quoted);
      }

      const message = document.createElement("div");
      message.className = "textwarden-message";
      message.textContent = match.shortMessage || match.message;
      item.appendChild(message);

      const buttons = document.createElement("div");
      buttons.className = "textwarden-buttons";
      for (const replacement of (match.replacements || []).slice(0, MAX_SUGGESTIONS)) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "textwarden-fix";
        button.textContent = replacement.value === "" ? "(remove)" : replacement.value;
        button.addEventListener("click", () => applyFix(match, replacement.value));
        buttons.appendChild(button);
      }
      item.appendChild(buttons);
      panel.appendChild(item);
    }
  }

  function teardown() {
    activeField = null;
    lastCheckedText = null;
    matches = [];
    clearTimeout(debounceTimer);
    if (badge) badge.style.display = "none";
    if (underlineLayer) underlineLayer.textContent = "";
    hidePanel();
  }

  // ---------------------------------------------------------------- wiring

  document.addEventListener("focusin", (event) => {
    const field = editableRoot(event.target);
    if (!field) return;
    if (field !== activeField) {
      teardown();
      activeField = field;
      scheduleCheck();
    }
  });

  document.addEventListener("focusout", (event) => {
    // Delay so clicks on the badge/panel (which preventDefault) don't tear down
    setTimeout(() => {
      if (document.activeElement && editableRoot(document.activeElement) === activeField) return;
      teardown();
    }, 150);
  });

  document.addEventListener("input", (event) => {
    if (activeField && editableRoot(event.target) === activeField) {
      hidePanel();
      scheduleCheck();
    }
  });

  const reposition = () => {
    if (!activeField || !badge) return;
    positionBadge();
    renderUnderlines();
    if (panel.style.display === "block") positionPanel();
  };
  window.addEventListener("scroll", reposition, { passive: true, capture: true });
  window.addEventListener("resize", reposition, { passive: true });
})();
