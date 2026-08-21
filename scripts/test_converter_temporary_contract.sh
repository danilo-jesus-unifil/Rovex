#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

grep -Fq 'use std::fs::OpenOptions;' src/converters/paths.rs
grep -Fq '.create_new(true)' src/converters/paths.rs
grep -Fq 'reserva_de_temporario_e_atomica_e_cria_placeholder' src/converters/tests.rs
grep -Fq '.arg("-y")' src/converters/process.rs
grep -Fq 'ffmpeg_pode_sobrescrever_placeholder_temporario_reservado' src/converters/process_tests.rs

retry_block=$(sed -n '/for ffprobe in ffprobe_paths/,/let attempt =/p' src/converters/pipeline.rs)
if grep -Fq 'remove_file(&temporary)' <<<"$retry_block"; then
    echo 'falha: retry remove a reserva antes do spawn' >&2
    exit 1
fi

grep -Fq 'Converter temporary contract' .github/workflows/ci.yml
grep -Fq 'run: ./scripts/test_converter_temporary_contract.sh' .github/workflows/ci.yml

printf '%s\n' 'Contrato de temporários aprovado: reserva atômica, placeholder preservado e saída FFmpeg controlada.'
