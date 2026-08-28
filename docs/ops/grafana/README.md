# sharecli Grafana provisioning (C05 L49)

C05 L49 (Dashboard coverage) ships as **provisioning-as-code** — no Grafana UI clicks required.

This is the canonical, per-lane Grafana configuration for sharecli.
The Grafana container's `--config` must mount this directory at
`/etc/grafana/provisioning/...` (or the operator's chosen equivalent).

---

## Layout

```
docs/ops/grafana/
├── README.md                                    — this file
├── provisioning/
│   ├── datasources/
│   │   └── prometheus.yaml                      — sharecli Prometheus datasource
│   ├── dashboards/
│   │   └── sharecli-providers.yaml              — dashboard provider manifest
│   └── manifests/
│       └── sharecli-c05-manifest.json           — C05 cluster metadata (audit trail)
└── dashboards/
    ├── sharecli-serve.json                      — RED HTTP + process gauges
    ├── sharecli-process.json                    — fleet process health
    └── sharecli-trace.json                      — OTel trace ingest
```

---

## How to deploy

### Local (one-liner)

```bash
just grafana-up
# or, manually:
docker run --rm -p 3000:3000 \
  -v "$PWD/docs/ops/grafana/provisioning:/etc/grafana/provisioning:ro" \
  -v "$PWD/docs/ops/grafana/dashboards:/var/lib/grafana/dashboards/sharecli:ro" \
  -e GF_SECURITY_ADMIN_PASSWORD=admin \
  grafana/grafana:11.3.0
```

### CI / cluster

Mount this directory at `/etc/grafana/provisioning` in the cluster's
Grafana deployment manifest. No additional configuration needed; the
provider manifest polls `dashboards/` every 30s for changes.

---

## Adding a new dashboard

1. Drop a `sharecli-<lane>.json` file in `docs/ops/grafana/dashboards/`.
2. (Optional) Add an entry in `provisioning/manifests/sharecli-<cluster>-manifest.json`
   so audit/RC can verify the dashboard is part of the wave's evidence.
3. Open a PR; on merge, the next Grafana container reload picks it up.

Do **not** edit dashboards via the Grafana UI — that bypasses this
git-tracked evidence path and breaks the L49 claim.

---

## Promoting a lane-level dashboard to org-wide

L49's gap explicitly mentions "lane/org" Grafana folders. The current
ship-shape is **lane-only** (the `sharecli` folder); promotion to
org-wide requires an additional `folder: sharecli-public` provider
in `provisioning/dashboards/sharecli-providers.yaml` plus org-internal
SSO scoping. That promotion is out of scope for Plan 782.

---

## Verification on first deploy

```bash
curl -u admin:admin http://localhost:3000/api/datasources | jq '.[]|.name'
# expect: "Prometheus"

curl -u admin:admin http://localhost:3000/api/search?query=sharecli | jq '.[].title'
# expect: ["sharecli serve", "sharecli process", "sharecli trace"]

curl -u admin:admin http://localhost:3000/api/folders | jq '.[]|select(.uid=="sharecli-folder")|.title'
# expect: "sharecli"
```

If any check fails, see `docs/ops/grafana/troubleshooting.md` (TODO).
