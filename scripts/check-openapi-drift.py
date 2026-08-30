#!/usr/bin/env python3
"""Fail if `sharecli serve` Axum routes drift from docs/openapi/serve.yaml (C00 L2).

Extracts `.route("…")` paths from src/commands/serve.rs and top-level path keys
from the OpenAPI YAML. Both sets must match exactly (symmetric diff).

Usage:
  python scripts/check-openapi-drift.py
  python scripts/check-openapi-drift.py --serve src/commands/serve.rs --openapi docs/openapi/serve.yaml
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROUTE_RE = re.compile(r"""\.route\(\s*["']([^"']+)["']""")
# OpenAPI path keys at indent 2 under `paths:` (e.g. `  /healthz:`).
PATH_KEY_RE = re.compile(r"^  (/[^:\s]*):\s*$", re.MULTILINE)


def extract_routes(serve_rs: Path) -> set[str]:
    text = serve_rs.read_text(encoding="utf-8")
    return set(ROUTE_RE.findall(text))


def extract_openapi_paths(openapi: Path) -> set[str]:
    text = openapi.read_text(encoding="utf-8")
    # Prefer a small dedicated parse over PyYAML dependency.
    paths_block = re.search(r"^paths:\n(.*?)(?=(?:^components:|$))", text, re.MULTILINE | re.DOTALL)
    if not paths_block:
        raise SystemExit(f"no paths: block in {openapi}")
    return set(PATH_KEY_RE.findall(paths_block.group(0)))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--serve", type=Path, default=Path("src/commands/serve.rs"))
    ap.add_argument("--openapi", type=Path, default=Path("docs/openapi/serve.yaml"))
    args = ap.parse_args()

    if not args.serve.is_file():
        print(f"missing serve source: {args.serve}", file=sys.stderr)
        return 2
    if not args.openapi.is_file():
        print(f"missing OpenAPI: {args.openapi}", file=sys.stderr)
        return 2

    routes = extract_routes(args.serve)
    documented = extract_openapi_paths(args.openapi)

    missing = sorted(routes - documented)
    extra = sorted(documented - routes)

    print(f"serve routes ({len(routes)}): {', '.join(sorted(routes))}")
    print(f"openapi paths ({len(documented)}): {', '.join(sorted(documented))}")

    if missing or extra:
        if missing:
            print("MISSING from OpenAPI (present in serve.rs):", file=sys.stderr)
            for p in missing:
                print(f"  - {p}", file=sys.stderr)
        if extra:
            print("EXTRA in OpenAPI (absent from serve.rs):", file=sys.stderr)
            for p in extra:
                print(f"  - {p}", file=sys.stderr)
        print(
            "OpenAPI drift detected — update docs/openapi/serve.yaml or serve routes.",
            file=sys.stderr,
        )
        return 1

    print("OpenAPI drift check OK (symmetric match).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
