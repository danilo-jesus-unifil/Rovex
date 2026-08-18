#!/usr/bin/env bash
set -euo pipefail

exe_path=${1:?uso: verify_windows_manifest.sh <rovex.exe>}
if [[ ! -f "$exe_path" ]]; then
    printf 'executável não encontrado: %s\n' "$exe_path" >&2
    exit 1
fi

resource_dump=$(mktemp)
trap 'rm -f "$resource_dump"' EXIT
objcopy --dump-section .rsrc="$resource_dump" "$exe_path"

for required_value in \
    'asInvoker' \
    'PerMonitorV2' \
    'PerMonitor' \
    'System' \
    'longPathAware' \
    'Microsoft.Windows.Common-Controls'; do
    if ! grep -a -F -q "$required_value" "$resource_dump"; then
        printf 'valor ausente no manifesto embutido: %s\n' "$required_value" >&2
        exit 1
    fi
done

printf 'manifesto Windows validado em %s\n' "$exe_path"
