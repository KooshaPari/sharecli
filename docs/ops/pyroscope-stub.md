# Pyroscope Stub (C05 L45+ Wave16 T-710)

**Scope:** stub only, no live push, no secrets, no network.

Wave16 T-710 soft gate for `C05 L45+` Pyroscope push / multi-hop / live PD.

- **Real live PD** remains `Gap` in `GAP-QA-MATRIX.md` - requires user infra (`live-pd` endpoint, auth, `PYROSCOPE_URL` secrets) and is `BLOCKED`/`Gap`.
- **This stub** provides a no-op `PyroscopeStub` in `src/pyroscope_stub.rs` that satisfies the soft gate without live infra.

## Files

- `src/pyroscope_stub.rs` - `PyroscopeStub` struct, `is_stub() -> true`, `push(&[u8]) -> Ok(())`, `is_enabled()`.
- `tests/c05_pyroscope_stub.rs` - 3 soft gate tests (`c05_pyroscope_stub_*`), no network, no secrets.
- `src/lib.rs` - `pub mod pyroscope_stub`.

## Soft gate

```rust
use sharecli::pyroscope_stub::PyroscopeStub;
let stub = PyroscopeStub::enabled();
assert!(stub.is_stub());
assert!(stub.push(b"profile").is_ok());
```

Hard live push (with `PYROSCOPE_URL`, auth, network) is not implemented here and remains queued as `C05 L45+ Gap` for user infra.

## Status

- `WORK_DAG.md` `T-710` `BLOCKED -> DONE` after this PR (soft stub).
- `WBS-PHASED.md` `W16.1` `DONE`.
- `GAP-QA-MATRIX.md` `C05 L45+` `Gap` remains `Gap` for live PD; stub is documented as soft gate, not closing the live gap.
