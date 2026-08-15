#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if ! command -v xvfb-run >/dev/null 2>&1; then
    printf '%s\n' 'xvfb-run não está instalado; smoke test gráfico não executado.'
    exit 0
fi
log_file="$(mktemp /tmp/rovex-smoke.XXXXXX.log)"
trap 'rm -f "$log_file"' EXIT
set +e
timeout --signal=TERM 12s xvfb-run -a cargo run --quiet -- /tmp >"$log_file" 2>&1
status=$?
set -e
cat "$log_file"
if [ "$status" -ne 124 ]; then
    printf 'Aplicativo encerrou antes do timeout esperado (status=%s).\n' "$status" >&2
    exit "$status"
fi
printf '%s\n' 'Smoke test gráfico concluído: o processo permaneceu ativo até o timeout esperado.'
