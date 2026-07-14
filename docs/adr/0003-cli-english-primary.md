# ADR 0003 — CLI localization policy (English-primary)

**Status:** Accepted  
**Date:** 2026-07-14  
**Deciders:** sharecli maintainers  
**Traceability:** audit-v38 C01 L16, `FUNCTIONAL_REQUIREMENTS.md`

## Context

audit-v38 L16 expects fluent/gettext-style i18n. sharecli is a local operator
CLI/daemon with English diagnostics, docs, and CI messages. Full localization
is not a current product requirement.

## Decision

1. **Primary locale:** American English for CLI help, errors, logs, and docs.
2. **Deferred:** fluent/gettext crates, translated catalogs, and locale
   negotiation until a documented multi-locale operator need appears.
3. **Seed:** keep user-facing strings centralized where practical; avoid
   scattering new hard-coded user prose without a path to extraction later.

## Consequences

- Auditors score L16 as **seeded / deferred** (not absent product defect).
- Revisit if sharecli ships a non-English operator surface or OEM localization.
