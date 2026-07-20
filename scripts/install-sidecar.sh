#!/usr/bin/env bash
# Install sharecli as a background sidecar service (KeepAlive HTTP serve).
# Usage: ./scripts/install-sidecar.sh [--uninstall]
# Bind defaults to 127.0.0.1:9000 (override with SHARECLI_SERVE_BIND).
set -euo pipefail

BINARY="sharecli"
SERVICE_NAME="com.phenotype.sharecli"
SERVE_BIND="${SHARECLI_SERVE_BIND:-127.0.0.1:9000}"

detect_os() {
  case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux)  echo "linux" ;;
    *)      echo "unsupported" ;;
  esac
}

ensure_state_dir() {
  mkdir -p "${HOME}/.sharecli"
}

install_macos() {
  local bin_path
  bin_path="$(which "$BINARY" 2>/dev/null || echo "")"
  if [[ -z "$bin_path" ]]; then
    echo "Error: '$BINARY' not found in PATH. Run 'cargo install --path .' first." >&2
    exit 1
  fi

  ensure_state_dir
  local plist_path="$HOME/Library/LaunchAgents/${SERVICE_NAME}.plist"

  # Unload any previous agent before rewriting the plist.
  launchctl bootout "gui/$(id -u)/${SERVICE_NAME}" 2>/dev/null || true
  launchctl unload "$plist_path" 2>/dev/null || true

  cat > "$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${SERVICE_NAME}</string>
  <key>ProgramArguments</key>
  <array>
    <string>${bin_path}</string>
    <string>serve</string>
    <string>--bind</string>
    <string>${SERVE_BIND}</string>
    <string>--on-conflict</string>
    <string>replace</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>WorkingDirectory</key>
  <string>${HOME}/.sharecli</string>
  <key>StandardOutPath</key>
  <string>${HOME}/.sharecli/sidecar.log</string>
  <key>StandardErrorPath</key>
  <string>${HOME}/.sharecli/sidecar.err</string>
</dict>
</plist>
EOF

  # Prefer modern bootstrap; fall back to load for older macOS.
  if launchctl bootstrap "gui/$(id -u)" "$plist_path" 2>/dev/null; then
    launchctl enable "gui/$(id -u)/${SERVICE_NAME}" 2>/dev/null || true
    launchctl kickstart -k "gui/$(id -u)/${SERVICE_NAME}" 2>/dev/null || true
  else
    launchctl load -w "$plist_path"
  fi
  echo "Installed KeepAlive serve LaunchAgent: $plist_path"
  echo "  bind=${SERVE_BIND}  binary=${bin_path}"
  echo "  health: curl -sf http://${SERVE_BIND}/healthz"
}

uninstall_macos() {
  local plist_path="$HOME/Library/LaunchAgents/${SERVICE_NAME}.plist"
  launchctl bootout "gui/$(id -u)/${SERVICE_NAME}" 2>/dev/null || true
  launchctl unload "$plist_path" 2>/dev/null || true
  rm -f "$plist_path"
  echo "Uninstalled: $plist_path"
}

install_linux() {
  local bin_path
  bin_path="$(which "$BINARY" 2>/dev/null || echo "")"
  if [[ -z "$bin_path" ]]; then
    echo "Error: '$BINARY' not found in PATH. Run 'cargo install --path .' first." >&2
    exit 1
  fi

  ensure_state_dir
  local service_file="$HOME/.config/systemd/user/${SERVICE_NAME}.service"
  mkdir -p "$(dirname "$service_file")"
  cat > "$service_file" <<EOF
[Unit]
Description=sharecli HTTP serve sidecar
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=%h/.sharecli
ExecStart=${bin_path} serve --bind ${SERVE_BIND} --on-conflict replace
Restart=always
RestartSec=2

[Install]
WantedBy=default.target
EOF
  systemctl --user daemon-reload
  systemctl --user enable --now "$SERVICE_NAME"
  echo "Installed KeepAlive serve unit: $service_file"
  echo "  bind=${SERVE_BIND}  binary=${bin_path}"
}

uninstall_linux() {
  systemctl --user disable --now "$SERVICE_NAME" 2>/dev/null || true
  rm -f "$HOME/.config/systemd/user/${SERVICE_NAME}.service"
  systemctl --user daemon-reload 2>/dev/null || true
  echo "Uninstalled: ${SERVICE_NAME}.service"
}

main() {
  local uninstall=false
  for arg in "$@"; do
    [[ "$arg" == "--uninstall" ]] && uninstall=true
  done

  local os
  os="$(detect_os)"
  case "$os" in
    macos)
      $uninstall && uninstall_macos || install_macos ;;
    linux)
      $uninstall && uninstall_linux || install_linux ;;
    *)
      echo "Error: unsupported OS '$(uname -s)'" >&2; exit 1 ;;
  esac
}

main "$@"
