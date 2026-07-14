#!/usr/bin/env node
/**
 * axe-core scan for src/dashboard.html (C09 L81.1 / FR-004 NFR).
 *
 * Static HTML via jsdom — no browser/ChromeDriver. Fails on serious/critical
 * violations for WCAG 2.x Level A tag groups.
 */
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { JSDOM } from 'jsdom';
import axe from 'axe-core';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '../..');
const dashboardPath = join(root, 'src/dashboard.html');

/** WCAG 2.x Level A tags scanned (documented in docs/a11y/README.md). */
const WCAG2A_TAGS = ['wcag2a', 'wcag21a', 'wcag22a'];
const BLOCKING_IMPACTS = new Set(['serious', 'critical']);

const html = readFileSync(dashboardPath, 'utf8');
const dom = new JSDOM(html, {
  url: 'http://127.0.0.1/dashboard.html',
});
const { document } = dom.window;

try {
  const results = await axe.run(document.documentElement, {
    runOnly: { type: 'tag', values: WCAG2A_TAGS },
    resultTypes: ['violations'],
  });

  const violations = results.violations ?? [];
  const blocking = violations.filter((v) => BLOCKING_IMPACTS.has(v.impact));
  const moderate = violations.filter((v) => !BLOCKING_IMPACTS.has(v.impact));

  console.log(
    `axe dashboard scan — tags: ${WCAG2A_TAGS.join(', ')}; violations: ${violations.length}`,
  );

  for (const v of violations) {
    const nodes = v.nodes?.length ?? 0;
    console.log(`  [${v.impact}] ${v.id} (${nodes} node(s)): ${v.help}`);
  }

  if (blocking.length > 0) {
    console.error(
      `\nFAIL: ${blocking.length} serious/critical violation(s) on ${dashboardPath}`,
    );
    process.exit(1);
  }

  if (moderate.length > 0) {
    console.log(
      `\nPASS (soft): ${moderate.length} moderate/minor violation(s) — not blocking CI`,
    );
  } else {
    console.log('\nPASS: zero axe violations for WCAG 2.x Level A tags');
  }
} finally {
  dom.window.close();
}
