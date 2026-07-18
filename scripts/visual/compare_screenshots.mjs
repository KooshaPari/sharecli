/**
 * Blocking pixel diff for dashboard PNG baselines (C10 L107 / T-600).
 *
 * Compares committed baselines under tests/visual/dashboard/ against fresh
 * Playwright captures (artifacts/playwright/*). Validates manifest byte sizes.
 *
 * Env:
 *   SHARECLI_PW_OUT — capture dir (default artifacts/playwright)
 *   UPDATE_VISUALS=1 — copy captures over baselines (regen; refresh manifest bytes)
 */
import { readFileSync, existsSync, statSync, copyFileSync } from "node:fs";
import { join, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const DASH_DIR = join(ROOT, "tests/visual/dashboard");
const ART_DIR = process.env.SHARECLI_PW_OUT || join(ROOT, "artifacts/playwright");

const manifest = JSON.parse(readFileSync(join(DASH_DIR, "manifest.json"), "utf8"));

const ART_BY_NAME = Object.fromEntries(
  manifest.baselines.map((b) => [b.name, basename(b.artifact_source)]),
);

if (process.env.UPDATE_VISUALS === "1") {
  for (const b of manifest.baselines) {
    const src = join(ART_DIR, ART_BY_NAME[b.name]);
    const dst = join(DASH_DIR, b.file);
    if (!existsSync(src)) {
      console.error(`missing capture for ${b.name}: ${src}`);
      process.exit(1);
    }
    copyFileSync(src, dst);
    console.log(`updated ${b.file} (${statSync(dst).size} bytes)`);
  }
  console.log("baselines refreshed — update manifest.json bytes before commit");
  process.exit(0);
}

let failed = false;

for (const b of manifest.baselines) {
  const baselinePath = join(DASH_DIR, b.file);
  const actualPath = join(ART_DIR, ART_BY_NAME[b.name]);

  if (!existsSync(baselinePath)) {
    console.error(`MISS baseline missing: ${b.file}`);
    failed = true;
    continue;
  }

  const bytes = statSync(baselinePath).size;
  if (b.bytes != null && bytes !== b.bytes) {
    console.error(`MISS ${b.file}: expected ${b.bytes} bytes, got ${bytes}`);
    failed = true;
  }

  if (!existsSync(actualPath)) {
    console.error(`MISS capture missing: ${ART_BY_NAME[b.name]} (run playwright_viewports.mjs)`);
    failed = true;
    continue;
  }

  const expected = PNG.sync.read(readFileSync(baselinePath));
  const actual = PNG.sync.read(readFileSync(actualPath));
  if (expected.width !== actual.width || expected.height !== actual.height) {
    console.error(
      `MISS ${b.name}: size ${expected.width}x${expected.height} vs ${actual.width}x${actual.height}`,
    );
    failed = true;
    continue;
  }

  const diffPixels = pixelmatch(expected.data, actual.data, null, expected.width, expected.height, {
    threshold: 0.1,
  });
  const pct = (diffPixels / (expected.width * expected.height)) * 100;
  const max = b.max_pixel_delta_pct ?? 0.2;
  if (pct > max) {
    console.error(`MISS ${b.name}: ${pct.toFixed(3)}% pixels differ (max ${max}%)`);
    failed = true;
  } else {
    console.log(`ok ${b.name}: ${pct.toFixed(3)}% delta, ${bytes} bytes (max ${max}%)`);
  }
}

process.exit(failed ? 1 : 0);
