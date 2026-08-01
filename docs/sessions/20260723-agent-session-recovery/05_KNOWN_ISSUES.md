# Known issues

- Existing `docs/session-recovery.md` documents `session recover` and watcher
  commands that current CLI dispatch does not implement.
- Ghostty currently has a capability shell and clipboard/window cast fallback,
  not native pane I/O or layout control.
- Session persistence is CRUD-oriented; it lacks automatic surface observation,
  harness/session-ID resolution, and a recovery executor.
- Current macOS FUSE work has semantic defects: the explicit FSKit request was
  removed while fallback messages still label an FSKit attempt; Unavailable
  still reaches fuser; the MFMount build change displaced Windows WinFsp build
  linkage.
- This host's MFMount probe reports that the FSKit file-system extension is not
  enabled. It must not block non-FUSE recovery work.

## Evidence checkpoint (2026-08-01 04:40 UTC)

The recovery artifact manifest was verified without cleanup, service startup, or
working-tree mutation:

```text
cd sharecli/recovery/feb-2026-agent-harness && sha256sum -c MANIFEST.sha256
all listed artifacts and configuration files: OK
```

Repository-wide `git diff --check` is also clean. Focused test execution is
currently deferred because the host has approximately 766 MiB free; the
cliproxy Go test attempt failed during setup with `no space left on device` in
`~/Library/Caches/go-build`. No cache cleanup was performed in this lane.
