# Fluent / gettext i18n roadmap (soft)

Audit-v38 **C01 L16** — internationalization catalogs deferred under English-primary policy.

## Current scope (shipped)

| Surface | Status | Evidence |
|---------|--------|----------|
| CLI English-primary policy | Done | [`docs/adr/0003-cli-english-primary.md`](../adr/0003-cli-english-primary.md) |
| User-facing CLI strings | English literals | `src/main.rs` clap `about` / `help` / subcommand text |
| Logs, errors, CI messages | English | `docs/ops/error-envelope.md`, audit JSONL schema |
| Dashboard / TUI | English | `docs/a11y/README.md`; thermal TUI crate |

sharecli is a local operator CLI/daemon. Full locale negotiation and translated catalogs are **not** a current product requirement. Auditors score L16 as **seeded / deferred** (ADR 0003), not an absent product defect.

## ADR 0003 — English-primary (cross-ref)

[`ADR 0003`](../adr/0003-cli-english-primary.md) accepts:

1. **Primary locale:** American English for CLI help, errors, logs, and docs.
2. **Deferred:** fluent/gettext crates, translated catalogs, and locale negotiation until a documented multi-locale operator need appears.
3. **Seed:** keep user-facing strings centralized where practical; avoid scattering new hard-coded user prose without a path to extraction later.

**Revisit trigger:** sharecli ships a non-English operator surface, OEM localization, or a GAP-QA-MATRIX promotion of L16 from **seeded** to **partial**.

## gettext vs Fluent (deferred hard)

When multi-locale becomes a product goal, pick **one** Rust-facing stack. Both are viable; sharecli bias is toward **Fluent** for asymmetric, natural-sounding CLI copy and **gettext** for POSIX/tooling familiarity.

| Criterion | **gettext** (`gettext-rs`, `.po`/`.mo`) | **Fluent** (`fluent`, `.ftl`) |
|-----------|----------------------------------------|--------------------------------|
| Ecosystem fit | GNU/Poedit/Weblate standard; familiar to distro packagers | Mozilla stack; strong for UI sentences with variants |
| Plural / gender | `ngettext` macros; mature but brittle for complex rules | Built-in selectors (`[count]`, gender) without reorder hacks |
| CLI ergonomics | `gettext!` / `ngettext!` at compile or runtime load | `fluent::FluentBundle` + message IDs; runtime locale bundles |
| Extraction | `xgettext` on `tr!()` markers | `fluent-syntax` + custom extractor or `cargo i18n` tooling |
| Runtime locale | `LC_MESSAGES`, `LANG`, `setlocale` | `LANG` / env override + explicit bundle load |
| Dashboard (if ever) | Separate web i18n (react-intl / next-intl) likely | Same `.ftl` files can feed Rust CLI + JS with `fluent-react` |
| Effort to adopt | M — marker pass + CI `.po` sync | M–L — bundle loader + FTL authoring workflow |

**Recommendation (soft):** defer crate choice until revisit. If only CLI ships first, **Fluent** aligns with asymmetric operator messages; if distro `.po` packaging is required, **gettext** wins on tooling.

## Soft wiring (this roadmap)

1. **No new i18n crates** on `main` until ADR 0003 is superseded or amended.
2. **String hygiene** — new user-facing CLI text goes through clap derives or shared modules; cross-ref [`error-envelope.md`](./error-envelope.md) for error copy consistency.
3. **Locale env (future)** — reserve `SHARECLI_LOCALE` or honor `LC_ALL` only after bundle loader exists; today unset → English.
4. **CI / golden** — `tests/golden/cli_help.txt` stays English; no locale matrix in CI until catalogs land.
5. **Dashboard** — C09 a11y docs remain English-primary; web i18n is a separate lane if product scope expands.

## Phased rollout (deferred hard)

| Phase | Deliverable | Hard gate? |
|-------|-------------|------------|
| **0 — today** | ADR 0003 + this roadmap | No |
| **1 — extract** | Message IDs for top-level CLI + top 20 errors; no translations | No |
| **2 — toolchain** | Pick gettext or Fluent; `locales/en-US` seed catalog; `just i18n-check` | No |
| **3 — second locale** | One pilot locale (e.g. `es` or `ja`) + contributor guide | No — opt-in |
| **4 — product** | Locale negotiation in CLI/TUI; Weblate/Crowdin sync; CI locale smoke | **Yes — deferred** |

Phase 0–3 are **documentation + soft prep** only. Phase 4 needs maintainer sign-off and ADR amendment.

## Extraction checklist (phase 1 — open)

| Area | Action | Status |
|------|--------|--------|
| `src/main.rs` clap metadata | Centralize `about` / `long_about` / subcommand help | Open |
| Error envelope user strings | Map codes → message IDs in `error-envelope.md` | Open |
| `sharecli status` / health text | English literals OK per ADR 0003 | Done (policy) |
| Thermal TUI | Defer until TUI i18n need documented | Deferred |
| Dashboard HTML | Out of C01 L16 CLI scope | Deferred |

## Commands (future — not wired)

```bash
# Placeholder; no task on main until phase 2
# just i18n-check    # validate FTL/.po + missing keys
# just i18n-extract  # regen template from Rust markers
```

## Audit evidence (C01 L16)

| Line | Evidence | Score |
|------|----------|-------|
| **L16** Internationalization | `docs/adr/0003-cli-english-primary.md`, this roadmap, English clap strings | **1** — seeded; catalogs deferred |

**Soft follow-up**

| Item | Status |
|------|--------|
| English-primary ADR | Done ([`0003`](../adr/0003-cli-english-primary.md)) |
| Fluent/gettext deferred roadmap | Done (this file) |
| gettext vs Fluent decision | Deferred |
| Catalog extraction + second locale | Deferred |

See also: [`0003-cli-english-primary.md`](../adr/0003-cli-english-primary.md), [`error-envelope.md`](./error-envelope.md), [`audit/.lane-c01/C01.md`](../../audit/.lane-c01/C01.md).
