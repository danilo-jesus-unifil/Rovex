#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# A conversão não pode voltar a controlar apenas o PID direto quando usa
# stdout/stderr encadeados: descendentes podem manter os pipes abertos.
grep -Fq 'command.process_group(0)' src/converters/process_tree.rs
grep -Fq 'libc::killpg' src/converters/process_tree.rs
grep -Fq 'CreateJobObjectW' src/converters/process_tree.rs
grep -Fq 'AssignProcessToJobObject' src/converters/process_tree.rs
grep -Fq 'JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE' src/converters/process_tree.rs
grep -Fq 'TerminateJobObject' src/converters/process_tree.rs
grep -Fq 'cancelamento_encerra_descendente_que_mantem_pipe_aberto' src/converters/process_tests.rs
grep -Fq 'Process containment contract' .github/workflows/ci.yml
grep -Fq 'run: ./scripts/test_process_containment_contract.sh' .github/workflows/ci.yml

grep -Fq 'ProcessTree::attach' src/converters/process.rs
grep -Fq 'terminate_child(&mut child, Some(&process_tree))' src/converters/process.rs

grep -Fq 'cargo test --all-targets --all-features' .github/workflows/ci.yml

printf '%s\n' 'Contrato de contenção aprovado: grupos/jobs, encerramento da árvore e regressão de pipe bloqueado cobertos.'
