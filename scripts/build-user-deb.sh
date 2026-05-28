#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEBUG_BIN="${ROOT_DIR}/target/debug/titrax-rs"
RELEASE_BIN="${ROOT_DIR}/target/release/titrax-rs"
OUT_DIR="${ROOT_DIR}/dist"
INSTALL_MODE=0
USE_RELEASE=0

usage() {
	cat <<EOF
Usage: $(basename "$0") [--release] [--install] [path-to-binary]

Builds a user-mode .deb that can be extracted into ~/.local without root.

Options:
  --release      Build from release binary (recommended for distribution)
  --install      Extract the built .deb into ~/.local after build
  -h, --help     Show this help

Binary resolution order when no path is given:
  1) target/release/titrax-rs  (if --release)
  2) target/debug/titrax-rs
  3) target/release/titrax-rs
  4) cargo build (debug)
EOF
}

SOURCE_BIN=""
while [[ $# -gt 0 ]]; do
	case "$1" in
	--release)
		USE_RELEASE=1
		shift
		;;
	--install)
		INSTALL_MODE=1
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		if [[ -n "${SOURCE_BIN}" ]]; then
			echo "error: multiple binary paths provided" >&2
			usage >&2
			exit 1
		fi
		SOURCE_BIN="$1"
		shift
		;;
	esac
done

if [[ -z "${SOURCE_BIN}" ]]; then
	if [[ ${USE_RELEASE} -eq 1 ]]; then
		if [[ ! -x "${RELEASE_BIN}" ]]; then
			cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"
		fi
		SOURCE_BIN="${RELEASE_BIN}"
	elif [[ -x "${DEBUG_BIN}" ]]; then
		SOURCE_BIN="${DEBUG_BIN}"
	elif [[ -x "${RELEASE_BIN}" ]]; then
		SOURCE_BIN="${RELEASE_BIN}"
	else
		cargo build --manifest-path "${ROOT_DIR}/Cargo.toml"
		SOURCE_BIN="${DEBUG_BIN}"
	fi
fi

if [[ ! -x "${SOURCE_BIN}" ]]; then
	echo "error: binary not found or not executable: ${SOURCE_BIN}" >&2
	exit 1
fi

if [[ ! -f "${ROOT_DIR}/share/icons/hicolor/64x64/apps/titrax-rs.png" ]]; then
	echo "error: missing icon file share/icons/hicolor/64x64/apps/titrax-rs.png" >&2
	exit 1
fi

version=$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
arch=$(dpkg --print-architecture 2>/dev/null || echo all)
pkg_name="titrax-rs-user"
deb_name="${pkg_name}_${version}_${arch}.deb"

build_root="$(mktemp -d /tmp/titrax-rs-user-deb-XXXXXX)"
trap 'rm -rf "${build_root}"' EXIT

mkdir -p "${build_root}/DEBIAN"
mkdir -p "${build_root}/bin"
mkdir -p "${build_root}/share/icons/hicolor/64x64/apps"
mkdir -p "${build_root}/share/applications"

cat >"${build_root}/DEBIAN/control" <<EOF
Package: ${pkg_name}
Version: ${version}
Section: utils
Priority: optional
Architecture: ${arch}
Maintainer: Titrax User Build <noreply@example.invalid>
Description: TimeTracker user-mode package for local extraction into ~/.local
EOF

install -m 755 "${SOURCE_BIN}" "${build_root}/bin/titrax-rs"
install -m 644 "${ROOT_DIR}/share/icons/hicolor/64x64/apps/titrax-rs.png" \
	"${build_root}/share/icons/hicolor/64x64/apps/titrax-rs.png"
cat >"${build_root}/share/applications/titrax-rs.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=TimeTracker (titrax-rs)
Comment=Track time on projects
Exec=titrax-rs
Icon=titrax-rs
Terminal=false
Categories=Office;Utility;
StartupNotify=true
EOF

mkdir -p "${OUT_DIR}"
dpkg-deb --build "${build_root}" "${OUT_DIR}/${deb_name}" >/dev/null

echo "Built: ${OUT_DIR}/${deb_name}"

if [[ ${INSTALL_MODE} -eq 1 ]]; then
	user_prefix="${HOME}/.local"
	dpkg-deb -x "${OUT_DIR}/${deb_name}" "${user_prefix}"

	if command -v gtk-update-icon-cache >/dev/null 2>&1; then
		gtk-update-icon-cache -q -t -f "${user_prefix}/share/icons/hicolor" || true
	fi
	if command -v update-desktop-database >/dev/null 2>&1; then
		update-desktop-database "${user_prefix}/share/applications" || true
	fi

	echo "Installed to: ${user_prefix}"
	echo "Binary: ${user_prefix}/bin/titrax-rs"
	echo "Desktop: ${user_prefix}/share/applications/titrax-rs.desktop"
fi
