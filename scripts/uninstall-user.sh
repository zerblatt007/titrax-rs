#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=0

usage() {
    cat <<EOF
Usage: $(basename "$0") [--dry-run]

Uninstalls user-mode Titrax files from ~/.local (or XDG overrides):
  - binary:   ~/.local/bin/titrax
  - desktop:  ~/.local/share/applications/titrax.desktop
  - icon:     ~/.local/share/icons/hicolor/64x64/apps/titrax.png

Options:
  --dry-run    Show what would be removed without deleting files
  -h, --help   Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

USER_BIN_DIR="${XDG_BIN_HOME:-${HOME}/.local/bin}"
USER_SHARE_DIR="${XDG_DATA_HOME:-${HOME}/.local/share}"
BIN_PATH="${USER_BIN_DIR}/titrax"
DESKTOP_PATH="${USER_SHARE_DIR}/applications/titrax.desktop"
ICON_PATH="${USER_SHARE_DIR}/icons/hicolor/64x64/apps/titrax.png"

remove_file() {
    local path="$1"
    if [[ -e "$path" ]]; then
        if [[ ${DRY_RUN} -eq 1 ]]; then
            echo "would remove: $path"
        else
            rm -f "$path"
            echo "removed: $path"
        fi
    else
        echo "not found: $path"
    fi
}

remove_file "$BIN_PATH"
remove_file "$DESKTOP_PATH"
remove_file "$ICON_PATH"

if [[ ${DRY_RUN} -eq 0 ]]; then
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f "${USER_SHARE_DIR}/icons/hicolor" || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "${USER_SHARE_DIR}/applications" || true
    fi
fi

echo "Done."
