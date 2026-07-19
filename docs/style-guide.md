# sharecli style guide — inclusive language & plain help

**Scope:** C09 L81.10 — user-visible CLI help, dashboard copy, and operator docs.

## Principles

1. **Plain language:** Prefer short sentences and common words. Target roughly 8th-grade reading level for help text and error messages.
2. **Inclusive language:** Avoid ableist, gendered, or culturally loaded terms in user-facing copy. Use person-first or neutral phrasing.
3. **Consistent naming:** Use `sharecli` (lowercase) for the binary; `ShareCLI` only in titles when needed.
4. **Action-oriented help:** `--help` lines start with a verb (`List`, `Start`, `Stop`) and describe outcome, not implementation.

## Avoid in user-visible strings

| Avoid | Prefer |
|-------|--------|
| blindly, crippled, idiot, stupid | (rephrase without judgment) |
| dumb terminal (non-technical) | plain terminal, non-ANSI terminal |
| master/slave (non-protocol) | primary/replica, leader/follower |
| guys, manpower | team, people, capacity |
| sanity check | quick check, smoke test |

**Technical exemptions:** Standard terms such as `TERM=dumb`, BIP39 word lists, and protocol names (e.g. OAuth) are allowed when they are industry-standard identifiers, not directed at people.

## CLI help conventions

- One sentence per subcommand description in `--help`.
- Flags document defaults and environment variables when non-obvious.
- Link to `docs/a11y/README.md` from top-level `--help` for accessibility posture.

## Automation

| Check | Command |
|-------|---------|
| Vale (docs) | `just vale` or `./scripts/lint/vale.sh` |
| Help golden | `cargo test --test golden_snapshots golden_cli_help` |
| Inclusive grep gate | `cargo test --test c09_l81_inclusive_language` |

CI: `.github/workflows/a11y.yml` job `vale-inclusive-language`.
