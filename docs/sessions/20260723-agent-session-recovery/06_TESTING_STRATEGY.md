# Testing strategy

- Unit-test capability state transitions, ledger migrations, record validation,
  resume recipe provenance, and bounded queue/backpressure behavior.
- Contract-test each terminal adapter with a mock process/transport runner.
- Use a disposable managed PTY integration fixture for output, input, crash, and
  recovery execution tests.
- Gate Ghostty-native integration tests behind explicit local capability probes;
  unsupported stable builds must produce a typed degraded result.
- Run FUSE KEXT, FSKit, and non-FUSE tests independently. Non-FUSE recovery is
  a required end-to-end gate on every platform.
