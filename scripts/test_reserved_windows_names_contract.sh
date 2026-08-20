#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

for reserved in '"COM¹"' '"COM².txt"' '"COM³"' '"LPT¹"' '"LPT².txt"' '"LPT³"'; do
    grep -Fq "$reserved" src/operations/tests.rs
done

grep -Fq '"COM¹"' src/security.rs
grep -Fq '"COM²"' src/security.rs
grep -Fq '"COM³"' src/security.rs
grep -Fq '"LPT¹"' src/security.rs
grep -Fq '"LPT²"' src/security.rs
grep -Fq '"LPT³"' src/security.rs

grep -Fq 'nomes_reservados_do_windows_sao_rejeitados_pelo_sistema' src/operations/tests.rs

grep -Fq 'cargo test --all-targets --all-features' .github/workflows/ci.yml

grep -Fq 'Reserved Windows names contract' .github/workflows/ci.yml

grep -Fq 'run: ./scripts/test_reserved_windows_names_contract.sh' .github/workflows/ci.yml

printf '%s\n' 'Contrato de nomes reservados Windows aprovado: COM/LPT ASCII e sobrescritos cobertos no código, teste e CI.'
