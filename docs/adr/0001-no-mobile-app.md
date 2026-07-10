# ADR 0001 — No mobile app

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** sharecli maintainers
- **Tags:** packaging, distribution, L117

## Context

Cluster C11 (packaging / distribution) asks whether sharecli should ship a
mobile presence (native iOS/Android, Expo, or PWA). The product is a **local
process supervisor and CLI toolkit**: it watches TOML configs, spawns OS
processes, exposes a loopback HTTP API (`/healthz`, `/readyz`, metrics), and
optionally a desktop tray. Operators run it on developer workstations and
fleet hosts — not on phones.

## Decision

**No mobile app, PWA, or responsive mobile web client.** Mobile is out of
scope for sharecli.

## Consequences

- Score L117 as a deliberate "no mobile, here's why" (not an unconsidered gap).
- Do not add Expo/React Native, Capacitor, or a service-worker PWA to this repo.
- Remote visibility, if needed later, belongs in a separate ops dashboard or
  fleet coordinator — not a phone-native shell around the supervisor.
- Revisit only if the product pivots to a hosted multi-tenant control plane
  with a first-class mobile operator UX.

## Alternatives considered

| Option | Why rejected |
| ------ | ------------ |
| Native iOS/Android | No process-spawn surface on mobile; App Store sandbox cannot host the supervisor model |
| Expo / React Native companion | Would duplicate the HTTP API without adding install/run value for the CLI |
| PWA wrapping `/` dashboard | Dashboard is localhost-bound; a PWA adds little over a browser bookmark |
| "Undecided / later" | Leaves L117 unscored; operator asked for an explicit answer |
