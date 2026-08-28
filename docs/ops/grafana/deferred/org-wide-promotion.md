# Why promotion to "org-wide" Grafana folders is deferred from C05 L49

L49's remaining gap is "Lane/org Grafana folders + provisioning as code".
Wave17 Plan 782 ships the **lane** half (the `sharecli` folder +
provisioning YAML + 3 dashboards). The **org-wide** half requires:

- Org-level SSO scoping (org admin approves cross-team folder visibility)
- A separate provider manifest under a different `orgId`
- Per-team SLAs for dashboard rotation (out of repo-local scope)

So Plan 782 closes the lane half but does not bump L49 → 4 (no such
score anyway; max is 3). L49 stays at 3 once `sharecli` folder is
in place. Promoting to org-wide is recorded as a future option but
does not increase L49's score.
