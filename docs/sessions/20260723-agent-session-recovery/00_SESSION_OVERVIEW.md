# Agent session recovery

Goal: recover agent conversations and live PTYs across Ghostty crashes without
requiring tmux, while preserving the daily Ghostty installation.

Ownership: ShareCLI owns process/surface control and local RPC; zmx owns managed
PTY continuity; SessionLedger owns durable conversation identity and transcripts.

Safety: stable Ghostty remains the default. Any native Ghostty build is a
separately signed canary. Ambiguous session matches are never auto-targeted.
