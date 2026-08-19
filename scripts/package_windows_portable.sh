#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
export TZ=UTC

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
TARGET=${1:-x86_64-pc-windows-gnu}
DIST_DIR=${2:-"$ROOT_DIR/dist"}
if [[ "$DIST_DIR" != /* ]]; then
    DIST_DIR="$ROOT_DIR/$DIST_DIR"
fi
VERSION=$(awk -F'"' '/^version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")
ARCHIVE_BASE="rovex-v${VERSION}-windows-x86_64-portable"
RELEASE_EXE="$ROOT_DIR/target/$TARGET/release/rovex.exe"

if [[ "$TARGET" != "x86_64-pc-windows-gnu" ]]; then
    printf 'target não suportado pelo pacote portable: %s\n' "$TARGET" >&2
    exit 2
fi

export PATH="${HOME}/.cargo/bin:$PATH"
cd "$ROOT_DIR"
cargo build --release --target "$TARGET"
test -f "$RELEASE_EXE"

mkdir -p "$DIST_DIR"
stage_dir=$(mktemp -d "$ROOT_DIR/.portable-stage.XXXXXX")
cleanup() {
    rm -rf "$stage_dir"
}
trap cleanup EXIT
package_dir="$stage_dir/$ARCHIVE_BASE"
mkdir -p "$package_dir"

cp "$RELEASE_EXE" "$package_dir/rovex.exe"
cp LICENSE README.md COMPATIBILITY.md "$package_dir/"
cp distribution/PORTABLE.txt "$package_dir/"

commit=$(git rev-parse HEAD)
cat > "$package_dir/DISTRIBUTION-MANIFEST.txt" <<EOF
Rovex portable distribution
version=$VERSION
target=$TARGET
profile=release
commit=$commit
signed=no
runtime_downloads=no
entrypoint=rovex.exe

This archive is unsigned. Verify the external SHA-256 file before extracting.
FFmpeg is optional and is not bundled; conversion features require user-provided executables.
EOF

# ZIP stores DOS timestamps; normalizing files makes repeated local packaging stable.
find "$package_dir" -exec touch -h -d '1980-01-01 00:00:00 UTC' {} +
archive="$DIST_DIR/$ARCHIVE_BASE.zip"
rm -f "$archive"
(
    cd "$stage_dir"
    zip -X -q -r "$archive" "$ARCHIVE_BASE"
)
(
    cd "$DIST_DIR"
    sha256sum "$(basename "$archive")" > "$ARCHIVE_BASE.sha256"
)
printf 'portable package: %s\n' "$archive"
printf 'sha256 manifest: %s\n' "$DIST_DIR/$ARCHIVE_BASE.sha256"
