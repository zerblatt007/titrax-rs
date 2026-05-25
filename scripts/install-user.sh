#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEBUG_BIN="${ROOT_DIR}/target/debug/titrax"
RELEASE_BIN="${ROOT_DIR}/target/release/titrax"

if [[ $# -gt 0 ]]; then
	SOURCE_BIN="$1"
elif [[ -x "${DEBUG_BIN}" ]]; then
	SOURCE_BIN="${DEBUG_BIN}"
elif [[ -x "${RELEASE_BIN}" ]]; then
	SOURCE_BIN="${RELEASE_BIN}"
else
	cargo build --manifest-path "${ROOT_DIR}/Cargo.toml"
	SOURCE_BIN="${DEBUG_BIN}"
fi

if [[ ! -x "${SOURCE_BIN}" ]]; then
	echo "error: binary not found or not executable: ${SOURCE_BIN}" >&2
	exit 1
fi

USER_BIN_DIR="${XDG_BIN_HOME:-${HOME}/.local/bin}"
USER_SHARE_DIR="${XDG_DATA_HOME:-${HOME}/.local/share}"
USER_ICON_DIR="${USER_SHARE_DIR}/icons/hicolor/64x64/apps"
USER_DESKTOP_DIR="${USER_SHARE_DIR}/applications"

mkdir -p "${USER_BIN_DIR}" "${USER_ICON_DIR}" "${USER_DESKTOP_DIR}"

install -m 755 "${SOURCE_BIN}" "${USER_BIN_DIR}/titrax"
install -m 644 "${ROOT_DIR}/share/icons/hicolor/64x64/apps/titrax.png" "${USER_ICON_DIR}/titrax.png"
cat > "${USER_DESKTOP_DIR}/titrax.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=TimeTracker
Comment=Track time on projects
Exec=${USER_BIN_DIR}/titrax
Icon=titrax
Terminal=false
Categories=Office;Utility;
StartupNotify=true
EOF

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
	gtk-update-icon-cache -q -t -f "${USER_SHARE_DIR}/icons/hicolor" || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
	update-desktop-database "${USER_DESKTOP_DIR}" || true
fi

echo "Installed Titrax for the current user:"
echo "  binary:  ${USER_BIN_DIR}/titrax"
echo "  icon:    ${USER_ICON_DIR}/titrax.png"
echo "  desktop: ${USER_DESKTOP_DIR}/titrax.desktop"
