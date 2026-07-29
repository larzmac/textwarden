// TextWarden Web — background service worker.
// Content scripts can't fetch http://localhost from https pages in Safari
// (mixed-content rules), so all LanguageTool requests are proxied through here.

const api = globalThis.browser ?? globalThis.chrome;

const LT_URL = "http://localhost:8081/v2/check";
const FAILURE_COOLDOWN_MS = 30_000;

let failedUntil = 0;

api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!message || message.type !== "lt-check") {
    return false;
  }

  if (Date.now() < failedUntil) {
    sendResponse({ error: "server-offline" });
    return false;
  }

  (async () => {
    try {
      const body = new URLSearchParams({
        text: message.text,
        language: message.language || "en-US",
      });
      const response = await fetch(LT_URL, { method: "POST", body });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      const data = await response.json();
      failedUntil = 0;
      sendResponse({ matches: data.matches || [] });
    } catch (error) {
      failedUntil = Date.now() + FAILURE_COOLDOWN_MS;
      sendResponse({ error: "server-offline" });
    }
  })();

  return true; // keep sendResponse alive for the async reply
});
