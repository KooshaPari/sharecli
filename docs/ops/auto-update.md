# Soft auto-update channel — sharecli

Audit-v38 **C11 L111**. No in-binary self-update / Sparkle / tauri-updater yet.

## Supported update paths (soft)

| Channel | How | Signature |
|---------|-----|-----------|
| crates.io | `cargo install sharecli --force` | crates.io / cargo audit |
| cargo-binstall | `cargo binstall sharecli` | release checksums when present |
| Homebrew | `brew upgrade sharecli` / `--HEAD` | bottle sha when filled |
| GitHub Releases | download latest `sharecli-*` + `.sha256` | checksum today; codesign L112 later |

## Not in scope yet

- In-process `sharecli self-update`
- Sparkle / WinUI auto-updater appcast
- Signature-verified delta updates (blocked on L112 secrets)

In-binary roadmap (TUF sketch, phases deferred): [`in-binary-updater.md`](in-binary-updater.md).

## Operator checklist

1. Prefer release artifacts or `cargo binstall` for binaries.
2. Verify `.sha256` before replacing a production binary.
3. After L112: prefer notarized/Authenticode builds only.
