#!/usr/bin/env node
/**
 * Playwright Tab-cycle for dashboard keyboard navigation (C09 L81.3 / FR-004 NFR).
 *
 * Requires a running `sharecli serve` (or SHARECLI_DASH_URL). Set
 * SHARECLI_VISUAL_FIXTURE=1 for deterministic WebSocket mock (CI).
 */
import { chromium } from "playwright";

const url = process.env.SHARECLI_DASH_URL || "http://127.0.0.1:9000/";
const deterministic = process.env.SHARECLI_VISUAL_FIXTURE === "1";

function fail(message) {
  console.error(`FAIL: ${message}`);
  process.exit(1);
}

const browser = await chromium.launch();
try {
  const context = await browser.newContext({
    colorScheme: "dark",
    locale: "en-US",
    reducedMotion: "reduce",
  });
  if (deterministic) {
    await context.addInitScript(() => {
      class VisualFixtureWebSocket extends EventTarget {
        static CONNECTING = 0;
        static OPEN = 1;
        static CLOSING = 2;
        static CLOSED = 3;

        CONNECTING = 0;
        OPEN = 1;
        CLOSING = 2;
        CLOSED = 3;

        constructor() {
          super();
          this.readyState = VisualFixtureWebSocket.CONNECTING;
          queueMicrotask(() => {
            this.readyState = VisualFixtureWebSocket.OPEN;
            const event = new Event("open");
            this.onopen?.(event);
            this.dispatchEvent(event);
          });
        }

        close() {
          this.readyState = VisualFixtureWebSocket.CLOSED;
        }

        send() {}
      }
      Object.defineProperty(window, "WebSocket", {
        configurable: false,
        value: VisualFixtureWebSocket,
        writable: false,
      });
    });
  }
  const page = await context.newPage();
  await page.goto(url, {
    waitUntil: deterministic ? "domcontentloaded" : "networkidle",
    timeout: 30000,
  });
  if (deterministic) {
    await page.waitForFunction(
      () => document.querySelector("#status-label")?.textContent === "connected",
      null,
      { timeout: 15000 },
    );
  }

  await page.keyboard.press("Tab");
  const skipFocused = await page.evaluate(
    () => document.activeElement?.classList?.contains("skip-link") ?? false,
  );
  if (!skipFocused) {
    fail("first Tab must focus the skip link");
  }

  const outlineOk = await page.evaluate(() => {
    const style = window.getComputedStyle(document.activeElement);
    const width = parseFloat(style.outlineWidth) || 0;
    return width >= 2;
  });
  if (!outlineOk) {
    fail("skip link must show a visible focus outline (≥2px)");
  }

  await page.keyboard.press("Enter");
  const mainFocused = await page.evaluate(() => {
    const main = document.getElementById("main-content");
    return document.activeElement === main;
  });
  if (!mainFocused) {
    fail("Enter on skip link must move focus to #main-content");
  }

  const tabOrder = [];
  const seen = new Set();
  for (let i = 0; i < 12; i += 1) {
    await page.keyboard.press("Tab");
    const token = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) {
        return "body";
      }
      const id = el.id || el.getAttribute("href") || el.className || "?";
      return `${el.tagName.toLowerCase()}#${id}`;
    });
    tabOrder.push(token);
    if (seen.has(token) && tabOrder.length > 1) {
      break;
    }
    seen.add(token);
  }

  console.log(`Tab order sample: ${tabOrder.join(" → ")}`);
  console.log("PASS: dashboard keyboard Tab-cycle");
} finally {
  await browser.close();
}
