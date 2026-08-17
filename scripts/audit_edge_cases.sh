#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
bin="${ROVEX_BIN:-target/release/rovex}"
root="$(mktemp -d /tmp/rovex-audit-edge.XXXXXX)"
cleanup() {
    chmod u+rwx "$root/no-access" 2>/dev/null || true
    rm -rf "$root"
}
trap cleanup EXIT

mkdir -p "$root/empty dir" "$root/space dir" "$root/no-access"
printf 'conteudo\n' > "$root/space dir/arquivo com espaço & acento.txt"
printf 'conteudo\n' > "$root/space dir/arquivo [especial].txt"
long_name="$(printf 'l%.0s' {1..180}).txt"
printf 'conteudo\n' > "$root/space dir/$long_name"
printf 'conteudo\n' > "$root/arquivo.txt"

empty_output="$($bin --cli "$root/empty dir")"
[[ "$empty_output" == *"Rovex core"* ]]
[[ "$empty_output" != *"[FILE]"* ]]

space_output="$($bin --cli "$root/space dir")"
[[ "$space_output" == *"arquivo com espaço & acento.txt"* ]]
[[ "$space_output" == *"arquivo [especial].txt"* ]]
[[ "$space_output" == *"$long_name"* ]]

if "$bin" --cli "$root/arquivo.txt" >"$root/not-directory.out" 2>&1; then
    printf '%s\n' 'falha: arquivo foi aceito como diretório' >&2
    exit 1
fi
grep -Fq 'não é um diretório' "$root/not-directory.out"

if "$bin" --cli "$root/missing" >"$root/missing.out" 2>&1; then
    printf '%s\n' 'falha: caminho inexistente foi aceito' >&2
    exit 1
fi
grep -Fq 'caminho não encontrado' "$root/missing.out"

chmod 000 "$root/no-access"
if "$bin" --cli "$root/no-access" >"$root/no-access.out" 2>&1; then
    printf '%s\n' 'falha: diretório sem permissão foi aceito' >&2
    exit 1
fi
if ! grep -Eq 'acesso negado|não foi possível' "$root/no-access.out"; then
    cat "$root/no-access.out" >&2
    exit 1
fi
chmod u+rwx "$root/no-access"

printf '%s\n' 'Casos extremos do CLI aprovados.'
