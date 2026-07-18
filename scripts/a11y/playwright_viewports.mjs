import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";

const url = process.env.SHARECLI_DASH_URL || "http://127.0.0.1:9000/";
const out = process.env.SHARECLI_PW_OUT || "artifacts/playwright";
const deterministic = process.env.SHARECLI_VISUAL_FIXTURE === "1";
mkdirSync(out, { recursive: true });

const viewports = [
  { name: "mobile-375", width: 375, height: 812 },
  { name: "tablet-768", width: 768, height: 1024 },
  { name: "desktop-1280", width: 1280, height: 800 },
];

const browser = await chromium.launch();
try {
  for (const vp of viewports) {
    const context = await browser.newContext({
      viewport: { width: vp.width, height: vp.height },
      colorScheme: "dark",
      deviceScaleFactor: 1,
      locale: "en-US",
      reducedMotion: "reduce",
      timezoneId: "UTC",
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
    await page.goto(url, { waitUntil: deterministic ? "domcontentloaded" : "networkidle", timeout: 30000 });
    if (deterministic) {
      await page.waitForFunction(
        () => document.querySelector("#status-label")?.textContent === "connected",
      );
    }
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({
      animations: "disabled",
      fullPage: true,
      path: join(out, `${vp.name}.png`),
      scale: "css",
    });
    await context.close();
    console.log(`ok ${vp.name}`);
  }
} finally {
  await browser.close();
}
