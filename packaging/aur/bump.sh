#!/usr/bin/env bash
# Update AUR metadata for a GitHub release tag.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PKGBUILD="$ROOT/packaging/aur/PKGBUILD"
SRCINFO="$ROOT/packaging/aur/.SRCINFO"
TAG="${1:?usage: bump.sh <version|vVersion>}"
VER="${TAG#v}"
URL="https://github.com/fireflylabss/fat/archive/refs/tags/v${VER}.tar.gz"

for _ in $(seq 1 12); do
  curl -fsI "$URL" >/dev/null 2>&1 && break
  sleep 5
done
SHA="$(curl -fsSL "$URL" | sha256sum | awk '{print $1}')"

sed -i "s/^pkgver=.*/pkgver=${VER}/" "$PKGBUILD"
sed -i "s/^pkgrel=.*/pkgrel=1/" "$PKGBUILD"
sed -i "s/^sha256sums=.*/sha256sums=('${SHA}')/" "$PKGBUILD"

cat > "$SRCINFO" <<EOF
pkgbase = ofat
	pkgdesc = Fast, syntax-aware cat alternative written in Rust
	pkgver = ${VER}
	pkgrel = 1
	url = https://github.com/fireflylabss/fat
	arch = x86_64
	license = Apache-2.0
	makedepends = cargo
	source = ofat-${VER}.tar.gz::https://github.com/fireflylabss/fat/archive/refs/tags/v${VER}.tar.gz
	sha256sums = ${SHA}

pkgname = ofat
EOF
