# Dependabot deferral — sharecli Wave 1

**Status:** DEFERRED — do not merge until Wave 1 is green  
**Date:** 2026-07-10  
**Wave 1 PR:** https://github.com/KooshaPari/sharecli/pull/202 (`feat/sharecli-wave1-lift`)

## Open Dependabot PRs (hold)

| PR | Title | Branch |
|----|-------|--------|
| [#193](https://github.com/KooshaPari/sharecli/pull/193) | chore(deps): bump reqwest from 0.12.28 to 0.13.4 | `dependabot/cargo/reqwest-0.13.4` |
| [#194](https://github.com/KooshaPari/sharecli/pull/194) | chore(deps): bump sha2 from 0.10.9 to 0.11.0 | `dependabot/cargo/sha2-0.11.0` |
| [#199](https://github.com/KooshaPari/sharecli/pull/199) | chore(deps): bump bytes from 1.12.0 to 1.12.1 | `dependabot/cargo/bytes-1.12.1` |
| [#200](https://github.com/KooshaPari/sharecli/pull/200) | chore(deps): bump runtime-process from `f921e62` to `696df07` | `dependabot/cargo/runtime-process-696df07` |
| [#201](https://github.com/KooshaPari/sharecli/pull/201) | chore(deps): bump substrate from `f921e62` to `696df07` | `dependabot/cargo/substrate-696df07` |

## Why

Wave 1 (#202) is still open and must stay mergeable / CI-green without unrelated Cargo bumps. Merging Dependabot into `main` (or rebasing Wave 1 onto them) risks lockfile churn and false CI noise while spawn-win / C07–C11 lifts land.

## Resume when

1. #202 is merged (or closed with Wave 1 accepted), **and**
2. `main` CI Success is green post-merge.

Then rebase/recreate the Dependabot PRs as needed and merge in a separate deps pass.
