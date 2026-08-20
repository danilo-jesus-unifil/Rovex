# Relatório de release — Rovex v0.1.25

**Data:** 2026-08-20

A v0.1.25 é uma release de robustez do ciclo de conversão. A auditoria confirmou por reprodução que o Rovex matava o processo FFmpeg/ffprobe direto, mas podia aguardar indefinidamente ou até o fim natural um leitor de stdout/stderr quando um descendente herdava o pipe. A correção agora termina a árvore de processos antes de aguardar seus leitores.

| Item | Resultado |
|---|---|
| Versão | `0.1.25` |
| Checkpoint | `backup/before-job-object-hardening-20260820` |
| Código | `src/converters/process_tree.rs` e `src/converters/process.rs` |
| Regressão | `cancelamento_encerra_descendente_que_mantem_pipe_aberto` |
| Gate | `scripts/test_process_containment_contract.sh` |
| Pesquisa | `docs/research/process-containment-research-2026-08-20.md` |

## Falha confirmada

O worker cria pipes para os diagnósticos do backend e inicia leitores em threads dedicadas. Em cancelamento, timeout ou erro de espera, a implementação anterior encerrava o `Child` direto e depois fazia `join` dos leitores. A documentação da Microsoft confirma que pipes podem ser herdados por processos filhos quando as condições de herança são atendidas.[1] Portanto, um backend que cria um descendente e mantém stderr/stdout aberto pode impedir que o leitor observe EOF após o processo direto morrer.

A hipótese foi reproduzida no host com um backend fake que inicia `sleep` em background, herda stderr e sinaliza um arquivo de prontidão. O teste passou somente depois de a terminação alcançar o grupo do processo; ele mede retorno rápido e não confunde uma espera natural de segundos com cancelamento bem-sucedido.

> O achado foi classificado como falha real porque houve reprodução executável no caminho efetivo do worker, não apenas análise estática de uma API.

## Correção implementada

Em Unix, `CommandExt::process_group(0)` coloca cada backend em um grupo próprio, e `libc::killpg` termina o grupo em cancelamento, timeout, falha de leitura ou falha de `try_wait`. Em Windows, o Rovex cria um Job Object, configura `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, associa o processo com `AssignProcessToJobObject` e usa `TerminateJobObject` nas rotas de interrupção. O RAII fecha o handle do job, preservando a política de encerramento da árvore.

A associação é feita imediatamente depois do spawn. Se a criação, configuração ou associação do Job Object falhar, a operação falha fechada depois de tentar terminar e aguardar o processo direto; o código não finge que a árvore está contida. O processo ainda recebe argumentos separados, stdin nulo e os mesmos limites de timeout, cancelamento, saída e validação de codec.

A Microsoft descreve Job Objects como grupos de processos administráveis como uma unidade e informa que filhos criados por `CreateProcess` são associados por padrão; também documenta `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` para terminar processos associados ao fechar o último handle.[2]

## Validação incrementada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all` | Aprovado |
| Teste de processos | 4 aprovados, incluindo a regressão de descendente |
| Suíte host prevista | 107 aprovados; 2 ignorados explicitamente |
| Clippy host | Aprovado com `-D warnings` |
| Check/Clippy Windows GNU | Aprovados com `-D warnings` |
| Contrato de contenção | Aprovado; grupos, Job Objects, terminação e teste exigidos |
| Contratos anteriores | Descoberta FFmpeg, ativação, Windows nativo e nomes reservados preservados |
| Modularidade | Todos os arquivos Rust abaixo de 400 linhas |

O job Windows do CI precisa validar a compilação, os testes existentes e os contratos nativos. A associação ao Job Object foi implementada com APIs do Windows já utilizadas pelo projeto, mas a cobertura nativa de um backend Windows que crie descendente ainda é uma validação futura recomendada.

## Riscos não declarados como resolvidos

Job Objects contêm processos criados normalmente pelo backend, mas não são sandboxing. O risco de DLL hijacking permanece: a Microsoft informa que dependências de uma DLL são procuradas por nome mesmo quando a primeira DLL foi carregada por caminho completo, e uma pasta pesquisada sob controle de atacante pode receber uma cópia maliciosa.[3] Autenticação por assinatura/hash, política de diretórios confiáveis, processos que tentam breakaway, corrida entre spawn e associação, TOCTOU de filesystem, ACLs, UNC/SMB, arquivos bloqueados, disco cheio, DPI, acessibilidade e execução interativa completa em Windows 10/11 continuam pendentes.

O próximo ciclo deve criar uma fixture Windows nativa para testar a associação real e documentar o comportamento quando o Rovex já estiver dentro de outro Job Object. Nenhuma dessas limitações é apresentada como corrigida nesta release.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/sysinfo/handle-inheritance "Microsoft — Handle Inheritance"
[2]: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects "Microsoft — Job Objects"
[3]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order "Microsoft — Dynamic-link library search order"
