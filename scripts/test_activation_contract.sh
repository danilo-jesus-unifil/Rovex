#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="${HOME}/.cargo/bin:${PATH}"

cargo test --lib activation::tests --all-features -- --nocapture

grep -Fq 'callback context-menu-open-requested();' ui/main.slint
grep -Fq 'callback open-requested;' ui/overlays.slint
grep -Fq 'callback open-with-requested;' ui/overlays.slint
grep -Fq 'ShellExecuteExW' src/activation.rs
grep -Fq 'SEE_MASK_NOASYNC' src/activation.rs
grep -Fq 'fMask: SEE_MASK_NOASYNC,' src/activation.rs
grep -Fq 'COINIT_DISABLE_OLE1DDE' src/activation.rs
grep -Fq 'lpVerb: std::ptr::null(),' src/activation.rs
grep -Fq 'lpFile: wide.as_ptr(),' src/activation.rs
grep -Fq 'lpParameters: std::ptr::null(),' src/activation.rs
grep -Fq 'lpDirectory: std::ptr::null(),' src/activation.rs
grep -Fq 'ShellExecuteFailed {' src/activation.rs
grep -Fq 'shell_error_description' src/activation.rs
if grep -Fq 'Command::new' src/activation.rs; then
    printf '%s\n' 'falha: ativação não pode construir comando externo' >&2
    exit 1
fi

printf '%s\n' 'Contrato de ativação explícita aprovado: validação, ShellExecuteExW sem DDE assíncrono e diagnóstico tipado, separado de Open With.'
