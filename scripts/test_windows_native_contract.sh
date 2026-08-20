#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

grep -Fq 'longPathAware' assets/rovex.manifest
grep -Fq '$longFile.Length -le 260' scripts/verify_windows_native.ps1
grep -Fq 'cargo run --quiet -- --cli $longDirectory' scripts/verify_windows_native.ps1
grep -Fq 'New-Item -ItemType Junction' scripts/verify_windows_native.ps1
grep -Fq 'CLI seguiu junction' scripts/verify_windows_native.ps1
grep -Fq 'if ($LASTEXITCODE -eq 0)' scripts/verify_windows_native.ps1
grep -Fq 'Native Windows CLI smoke' .github/workflows/ci.yml
grep -Fq 'run: ./scripts/verify_windows_native.ps1' .github/workflows/ci.yml

printf '%s\n' 'Contrato do smoke Windows aprovado: longPathAware, fixture > MAX_PATH, junction recusada e execução no CI.'
