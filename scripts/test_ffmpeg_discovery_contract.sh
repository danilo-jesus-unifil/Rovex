#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# O CWD só pode entrar se o usuário o declarar explicitamente no PATH ou em um
# override absoluto; backend_candidates não deve adicioná-lo como fallback oculto.
if grep -Fq 'current_directory' src/converters/backend.rs || grep -Fq 'SearchPathW' src/converters/backend.rs || grep -Fq 'windows_where_candidates' src/converters/backend.rs; then
    printf '%s\n' 'Falha: backend_candidates voltou a usar uma busca implícita no diretório atual.' >&2
    exit 1
fi

grep -Fq 'current_exe()' src/converters/backend.rs
grep -Fq 'adjacent_directory' src/converters/backend.rs
grep -Fq 'path.is_absolute()' src/converters/backend.rs
grep -Fq 'spawn_ffmpeg(&ffmpeg' src/converters/pipeline.rs

grep -Fq 'cargo test --all-targets --all-features' .github/workflows/ci.yml
grep -Fq 'FFmpeg discovery contract' .github/workflows/ci.yml
grep -Fq 'run: ./scripts/test_ffmpeg_discovery_contract.sh' .github/workflows/ci.yml

printf '%s\n' 'Contrato de descoberta FFmpeg aprovado: sem CWD implícito, candidatos absolutos e fallback protegido no CI.'
