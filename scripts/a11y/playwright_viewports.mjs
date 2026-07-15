import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";

const url = process.env.SHARECLI_DASH_URL || "http://127.0.0.1:9000/";
const out = process.env.SHARECLI_PW_OUT || "artifacts/playwright";
mkdirSync(out, { recursive: true });

const viewports = [
  { name: "mobile-375", width: 375, height: 812 },
  { name: "tablet-768", width: 768, height: 1024 },
  { name: "desktop-1280", width: 1280, height: 800 },
];

const browser = await chromium.launch();
try {
  for (const vp of viewports) {
    const page = await browser.newPage({ viewport: { width: vp.width, height: vp.height } });
    await page.goto(url, { waitUntil: "networkidle", timeout: 30000 });
    await page.screenshot({ path: join(out, `${vp.name}.png`), fullPage: true });
    await page.close();
    console.log(`ok ${vp.name}`);
  }
} finally {
  await browser.close();
}
