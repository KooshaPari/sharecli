# Container runtime hardening (C04 L40)

The `Containerfile` runs as non-root `USER sharecli` (uid 1000) with a
`HEALTHCHECK` on `/healthz`.

## Recommended `docker run` flags

```bash
docker run --rm \
  --read-only \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --security-opt no-new-privileges:true \
  --cap-drop ALL \
  -p 127.0.0.1:9000:9000 \
  sharecli:local serve --bind 0.0.0.0:9000
```

| Flag | Intent |
|------|--------|
| `--read-only` | Immutable rootfs |
| `--tmpfs /tmp` | Writable scratch without persistent RW layer |
| `no-new-privileges` | Block privilege escalation |
| `--cap-drop ALL` | Drop Linux capabilities |

Kernel seccomp custom profiles and rootless Podman are operator-optional;
document here so CI/docs evidence the sandbox posture beyond `USER`.
