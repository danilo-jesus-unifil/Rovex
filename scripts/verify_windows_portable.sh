#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
ARCHIVE=${1:?uso: verify_windows_portable.sh arquivo.zip [arquivo.sha256]}
CHECKSUM=${2:-"${ARCHIVE%.zip}.sha256"}
ARCHIVE=$(cd "$(dirname "$ARCHIVE")" && pwd)/$(basename "$ARCHIVE")
CHECKSUM=$(cd "$(dirname "$CHECKSUM")" && pwd)/$(basename "$CHECKSUM")

[[ -f "$ARCHIVE" ]] || { printf 'ZIP não encontrado: %s\n' "$ARCHIVE" >&2; exit 1; }
[[ -f "$CHECKSUM" ]] || { printf 'checksum não encontrado: %s\n' "$CHECKSUM" >&2; exit 1; }
(
    cd "$(dirname "$ARCHIVE")"
    sha256sum -c "$(basename "$CHECKSUM")"
)

extract_dir=$(mktemp -d)
cleanup() {
    rm -rf "$extract_dir"
}
trap cleanup EXIT

mapfile -t entries < <(unzip -Z1 "$ARCHIVE")
[[ "${#entries[@]}" -gt 0 ]] || { printf 'ZIP vazio\n' >&2; exit 1; }
for entry in "${entries[@]}"; do
    [[ "$entry" != /* && "$entry" != *'../'* && "$entry" != *'/..'* ]] || {
        printf 'entrada insegura no ZIP: %s\n' "$entry" >&2
        exit 1
    }
done
unzip -q "$ARCHIVE" -d "$extract_dir"

archive_base=$(basename "$ARCHIVE" .zip)
root_entry=${entries[0]%%/*}
package_dir="$extract_dir/$root_entry"
[[ -d "$package_dir" ]] || { printf 'raiz de pacote ausente\n' >&2; exit 1; }
[[ "$root_entry" == "$archive_base" ]] || {
    printf 'raiz do pacote não corresponde ao nome do ZIP: %s != %s\n' "$root_entry" "$archive_base" >&2
    exit 1
}
for required in rovex.exe LICENSE README.md COMPATIBILITY.md PORTABLE.txt DISTRIBUTION-MANIFEST.txt; do
    [[ -f "$package_dir/$required" ]] || {
        printf 'arquivo obrigatório ausente: %s\n' "$required" >&2
        exit 1
    }
done
expected_version=${archive_base#rovex-v}
expected_version=${expected_version%-windows-x86_64-portable}
grep -Fxq "version=$expected_version" "$package_dir/DISTRIBUTION-MANIFEST.txt" || {
    printf 'versão do manifesto não corresponde ao artefato: esperado %s\n' "$expected_version" >&2
    exit 1
}
grep -Fxq 'target=x86_64-pc-windows-gnu' "$package_dir/DISTRIBUTION-MANIFEST.txt" || {
    printf 'target Windows GNU ausente ou divergente no manifesto\n' >&2
    exit 1
}

grep -q '^signed=no$' "$package_dir/DISTRIBUTION-MANIFEST.txt"
grep -q '^runtime_downloads=no$' "$package_dir/DISTRIBUTION-MANIFEST.txt"
"$ROOT_DIR/scripts/verify_windows_manifest.sh" "$package_dir/rovex.exe"
printf 'portable validado: %s\n' "$ARCHIVE"
