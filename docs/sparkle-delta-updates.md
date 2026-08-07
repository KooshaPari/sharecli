# Sparkle delta updates — sharecli tray release process

This document explains how the `ShareCLITray.app` macOS auto-updater
receives signed, delta-aware releases. It complements the channel
picker shipped in t-69 (`Sources/ShareCLITray/ChannelPicker.swift`)
and the appcast templates under `docs/appcast-template.xml`.

## Audience

- Release engineers cutting a `sharecli` tag and uploading artefacts.
- macOS maintainers who need to rotate the Sparkle EdDSA key or
  rebuild the per-channel feeds.

## Concepts

Sparkle updates `ShareCLITray.app` by consulting an XML "appcast"
feed whose entries describe each published version. The tray ships
one appcast per release channel (stable / beta / alpha); the
`ChannelPicker` UI flips `SPUUpdater`'s feed URL between them via
`SPUUpdaterDelegate.feedURLString(for:)`.

Two update shapes are supported:

| Shape | Tag in appcast | Payload | When used |
|-------|---------------|---------|-----------|
| Full archive | `<enclosure>` | entire `.zip` / `.dmg` | new users, large jumps |
| Binary delta | `<sparkle:deltas><enclosure sparkle:deltaFrom="N"/></sparkle:deltas>` | bsdiff-style patch between two full bundles | every existing user (default) |

Delta updates are computed by Sparkle's `generate_appcast` tool
(`desktop/ShareCLITray/.build/checkouts/Sparkle/generate_appcast`).
Each delta is signed with the same EdDSA key used for the full
archive so Sparkle can verify the patch before applying it.

## One-time setup (per release engineer)

1. Generate an Ed25519 keypair via Sparkle's `generate_keys` tool:

   ```sh
   cd desktop/ShareCLITray/.build/checkouts/Sparkle
   swift run -c release generate_keys
   ```

   This stores the private key under the `https://sparkle-project.org`
   service in your login keychain (account: `ed25519`). The public
   key (`<key>.pub`) is the value that goes into the tray's
   `Info.plist` as `SUPublicEDKey`.

2. Wire the public key into the bundle. In CI this happens in
   `scripts/notarize-tray-macos.sh`; for local builds edit
   `desktop/ShareCLITray/Sources/ShareCLITray/Info.plist`:

   ```xml
   <key>SUPublicEDKey</key>
   <string>BASE64_EDDSA_PUBLIC_KEY</string>
   ```

3. Decide the channel feed URL prefix (production replaces
   `https://sharecli.example` with the real origin) and export:

   ```sh
   export SHARECLI_DOWNLOAD_PREFIX="https://sharecli.example/downloads"
   ```

## Cutting a release

1. Tag the repo:

   ```sh
   git tag -s -m "sharecli 0.3.1" v0.3.1
   git push origin v0.3.1
   ```

   The tag drives CI (`.github/workflows/release.yml`) which builds,
   notarises, and uploads the signed `.zip` per channel into the
   download-prefix bucket.

2. Populate the archives directory that `generate_appcast` reads.
   CI writes the new artefact to something like:

   ```text
   dist/appcast/archives/ShareCLITray-0.3.1.zip
   dist/appcast/archives/ShareCLITray-0.3.1.md   # release notes (optional)
   dist/appcast/archives/ShareCLITray-0.3.0.zip # previous version, kept for delta
   ```

   Keeping at least the previous version in `archives/` lets
   `generate_appcast` compute the delta automatically.

3. Generate and sign the appcast:

   ```sh
   ./scripts/build-appcast.sh --channel stable
   ./scripts/build-appcast.sh --channel beta
   ./scripts/build-appcast.sh --channel alpha
   ```

   The script invokes Sparkle's `generate_appcast` (building it
   locally if necessary) with the matching `--channel` value so each
   feed is tagged with the right `sparkle:channel` element. The
   output files are written to `dist/appcast/appcast-{channel}.xml`.

4. Promote the per-channel feeds to the origin (whatever hosts
   `https://sharecli.example`). Sparkle re-signs the feed on each
   generation so manual edits after this step must be followed by
   another `build-appcast.sh` invocation.

5. Smoke-check a single channel locally:

   ```sh
   SHARECLI_DOWNLOAD_PREFIX="file://$PWD/dist/appcast/archives" \
       ./scripts/install-tray-macos.sh --system
   ```

   The bundled `Contents/Resources/appcast-stable.xml` (and its
   per-channel siblings) should now reflect the new release.

## How binary deltas work

Sparkle's `generate_appcast` walks the archives directory and, for
each pair of consecutive `.zip` bundles, runs the `BinaryDelta`
helper to compute a bsdiff-style patch. The resulting `.delta` file
is dropped next to the archive and an enclosure is added inside a
`<sparkle:deltas>` block:

```xml
<sparkle:deltas>
  <enclosure url=".../ShareCLITray-0.3.1-from-0.3.0.delta"
             sparkle:deltaFrom="30"
             length="612384"
             type="application/octet-stream"
             sparkle:edSignature="..." />
</sparkle:deltas>
```

On the client side Sparkle:

1. Downloads the delta.
2. Verifies the EdDSA signature with `SUPublicEDKey`.
3. Reads the current app's bundle version, finds the matching
   `sparkle:deltaFrom`, and patches in place.
4. Falls back to the full archive if the patch can't be applied
   (corrupt delta, mismatched signature, unsupported prior version).

The default cap is `--maximum-deltas 5`; older priors get the full
archive instead.

## Invoking `generate_appcast` directly

`scripts/build-appcast.sh` wraps the tool, but you can drive it
yourself when iterating on feed shape:

```sh
cd desktop/ShareCLITray/.build/checkouts/Sparkle
swift run -c release generate_appcast \
    --channel stable \
    --download-url-prefix "https://sharecli.example/downloads" \
    -o /tmp/appcast-stable.xml \
    /path/to/archives
```

Useful flags:

| Flag | Purpose |
|------|---------|
| `--channel <name>` | Tags new items with `<sparkle:channel>` (stable / beta / alpha). |
| `--download-url-prefix <url>` | Prefix baked into every `<enclosure url>`. |
| `--maximum-deltas N` | Cap on deltas per update (default 5). |
| `--maximum-versions N` | How many versions to retain per branch (default 3). |
| `--ed-key-file -` | Read private EdDSA key from stdin (for CI secrets). |
| `--auto-prune-update-files` | Move old archives to `old_updates/` after 2 weeks. |

The script `scripts/build-appcast.sh` exposes `--channel`, `--install`,
`--archives`, `--out`, and `--prefix` so the same operations work
without rebuilding Sparkle.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `Error: Private key for account ed25519 not found in the Keychain` | first-time setup | run `generate_keys` once locally and unlock keychain in CI |
| `sparkle:edSignature="UNSIGNED_FALLBACK_REQUIRES_GENERATE_APPCAST"` | fallback emitter ran | install the Sparkle toolchain (`xcode-select --install` then `swift run generate_appcast`) |
| Tray reports "Up to date" against a freshly published release | wrong channel in `<sparkle:channel>` | re-run `build-appcast.sh --channel stable` |
| Delta patch fails to apply | `sparkle:deltaFrom` mismatch | verify the prior bundle version in `CFBundleVersion` matches the `<sparkle:deltaFrom>` integer |

## Related files

- `desktop/ShareCLITray/Sources/ShareCLITray/UpdaterView.swift` —
  Sparkle integration (SPUStandardUpdaterController + per-channel feed
  delegate).
- `desktop/ShareCLITray/Sources/ShareCLITray/ChannelPicker.swift` —
  channel picker UI and feed URL derivation.
- `docs/appcast-template.xml` — canonical appcast shape (stable channel
  with one `<sparkle:deltas>` example).
- `scripts/build-appcast.sh` — channel-aware appcast generator.
- `scripts/install-tray-macos.sh` — bundles the per-channel feeds
  into the `.app`.
