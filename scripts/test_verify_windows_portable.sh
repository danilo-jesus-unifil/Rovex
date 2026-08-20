#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
archive=${1:?uso: test_verify_windows_portable.sh arquivo.zip [verificador]}
verifier=${2:-scripts/verify_windows_portable.sh}
[[ -f "$archive" ]] || { printf 'ZIP de teste não encontrado: %s\n' "$archive" >&2; exit 1; }
[[ -x "$verifier" ]] || { printf 'verificador não executável: %s\n' "$verifier" >&2; exit 1; }

workspace=$(mktemp -d /tmp/rovex-portable-verifier-test.XXXXXX)
cleanup() {
    rm -rf "$workspace"
}
trap cleanup EXIT

unzip -q "$archive" -d "$workspace/original"
root_entry=$(find "$workspace/original" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | head -1)
[[ -n "$root_entry" ]] || { printf 'raiz do fixture não encontrada\n' >&2; exit 1; }

make_tampered_archive() {
    local name="$1"
    local mutation="$2"
    rm -rf "$workspace/case"
    mkdir -p "$workspace/case"
    cp -a "$workspace/original/$root_entry" "$workspace/case/$root_entry"
    case "$mutation" in
        version)
            sed -i 's/^version=[^$]*/version=9.9.9/' "$workspace/case/$root_entry/DISTRIBUTION-MANIFEST.txt"
            ;;
        target)
            sed -i 's/^target=[^$]*/target=unexpected-target/' "$workspace/case/$root_entry/DISTRIBUTION-MANIFEST.txt"
            ;;
        root)
            mv "$workspace/case/$root_entry" "$workspace/case/rovex-tampered-root"
            ;;
        *)
            printf 'mutação desconhecida: %s\n' "$mutation" >&2
            exit 1
            ;;
    esac
    (cd "$workspace/case" && zip -X -q -r "$workspace/$name.zip" .)
    (cd "$workspace" && sha256sum "$name.zip" > "$name.sha256")
}

expect_rejection() {
    local name="$1"
    if "$verifier" "$workspace/$name.zip" "$workspace/$name.sha256" >"$workspace/$name.log" 2>&1; then
        printf 'falha: verificador aceitou fixture adulterado (%s)\n' "$name" >&2
        cat "$workspace/$name.log" >&2
        exit 1
    fi
    printf 'PASS rejeição: %s\n' "$name"
}

"$verifier" "$archive" >/dev/null
make_tampered_archive version version
expect_rejection version
make_tampered_archive target target
expect_rejection target
make_tampered_archive root root
expect_rejection root
printf '%s\n' 'Teste adversarial do verificador portable aprovado.'
