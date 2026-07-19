# sharecli — sample systemd unit (C11 L115)

Audit-v38 **C11 L115**. Packaged in unsigned `.deb` artifacts at
`lib/systemd/system/sharecli.service` (see `scripts/packaging/build_deb.sh`).

Inspect packaged unit:

```bash
dpkg-deb -c dist/sharecli_*.deb | grep sharecli.service
```

```ini
[Unit]
Description=sharecli process supervisor (HTTP dashboard)
Documentation=https://github.com/KooshaPari/sharecli
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=sharecli
Group=sharecli
WorkingDirectory=/var/lib/sharecli
ExecStart=/usr/local/bin/sharecli serve --bind 127.0.0.1:9000
Restart=on-failure
RestartSec=2
# Optional bearer AuthN:
# Environment=SHARECLI_SERVE_TOKEN=replace-me
# Harden lightly:
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/sharecli

[Install]
WantedBy=multi-user.target
```

Install sketch:

```bash
sudo useradd --system --home /var/lib/sharecli --shell /usr/sbin/nologin sharecli
sudo install -d -o sharecli -g sharecli /var/lib/sharecli
sudo install -m 0644 docs/deploy/systemd/sharecli.service /etc/systemd/system/sharecli.service
sudo systemctl daemon-reload
sudo systemctl enable --now sharecli
curl -fsS http://127.0.0.1:9000/healthz
```

Reverse-proxy companion: [`../caddy/Caddyfile.sample`](../caddy/Caddyfile.sample).
