# Relatório de release — Rovex v0.1.26

**Data:** 2026-08-20

A v0.1.26 é uma release de endurecimento incremental do ciclo de conversão. A auditoria do código da v0.1.25 encontrou uma lacuna após a implementação de grupos Unix e Job Objects Windows: o resultado de `killpg`/`TerminateJobObject` era ignorado antes de `wait`. Se a API de terminação falhasse, o fluxo poderia voltar a depender somente do encerramento do processo direto, justamente quando um descendente mantivesse um pipe aberto.

| Item | Resultado |
|---|---|
| Versão | `0.1.26` |
| Checkpoint | `backup/before-job-object-hardening-20260820` preserva a base anterior; a correção foi feita em commit separado |
| Código | `src/converters/process_tree.rs` e `src/converters/process.rs` |
| Regressão | `cancelamento_encerra_descendente_que_mantem_pipe_aberto` |
| Gate | `scripts/test_process_containment_contract.sh` |
| Pesquisa | `docs/research/process-containment-research-2026-08-20.md` |

## Falha confirmada e classificação

A falha principal de pipes bloqueados já havia sido reproduzida na v0.1.25 com um backend fake Unix que inicia um descendente em background, herda stderr e continua executando. A v0.1.26 revisou o caminho de erro introduzido pela correção: ignorar o retorno da API de terminação e aguardar silenciosamente não fornecia um fallback explícito caso a terminação do grupo/job falhasse.

A Microsoft documenta que Job Objects administram grupos de processos como uma unidade e que `TerminateJobObject` termina os processos associados; também documenta que handles herdáveis podem incluir pipes.[1] [2] A consequência de ignorar uma falha de terminação foi classificada como uma lacuna real de robustez, embora a falha da API em um Windows normal não tenha sido reproduzida no runner. A correção não afirma que essa condição rara foi observada no Windows; ela garante que o código não descarta o erro silenciosamente.

## Correção implementada

`ProcessTree::terminate` agora retorna `io::Result<()>` em Unix, Windows e na implementação fallback. O helper comum verifica o resultado: se `killpg` ou `TerminateJobObject` falhar, chama `Child::kill` como último recurso e só então aguarda o processo direto. A configuração continua falhando fechada quando a criação, configuração ou associação do Job Object falha; o Rovex não declara contenção que não conseguiu estabelecer.

O comportamento principal permanece inalterado quando as APIs funcionam: grupos próprios em Unix, Job Object com `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` no Windows, stdin nulo, argumentos separados, readers limitados, timeout, cancelamento e validação pelo ffprobe. A mudança é pequena e não introduz uma segunda thread, shell ou caminho de execução.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all` | Aprovado |
| Testes de processo | 4 aprovados, incluindo descendente com pipe aberto |
| Suíte host | 107 aprovados; 2 ignorados explicitamente |
| Clippy host | Aprovado com `-D warnings` |
| Check/Clippy Windows GNU | Aprovados com `-D warnings` |
| Contrato de contenção | Aprovado e executado no CI |
| Auditoria de dependências | `cargo audit` e `cargo deny check` aprovados |
| CI Windows | Testes, build release, contratos e smoke nativo aprovados |

## Limitações honestas

O fallback `Child::kill` é somente uma tentativa final: se a terminação do grupo/job falhar, ele não garante que descendentes terminem. Por isso, a operação não transforma a falha em sucesso silencioso e o relatório não classifica essa condição como contenção completa. Uma fixture Windows nativa que injete falha de Job Object ou execute dentro de outro Job Object continua recomendada.

Job Objects não são sandboxing e não autenticam executáveis ou DLLs. A Microsoft informa que DLLs dependentes continuam sendo procuradas por nome mesmo quando a DLL inicial é carregada por caminho completo, e diretórios pesquisados sob controle de atacante podem receber cópias maliciosas.[3] Permanecem pendentes autenticação por assinatura/hash, política de DLLs confiáveis, processos que tentam breakaway, a janela entre spawn e associação, TOCTOU de filesystem, ACLs, UNC/SMB, arquivos bloqueados, disco cheio, DPI, acessibilidade e execução gráfica interativa completa em Windows 10/11.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects "Microsoft — Job Objects"
[2]: https://learn.microsoft.com/en-us/windows/win32/sysinfo/handle-inheritance "Microsoft — Handle Inheritance"
[3]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order "Microsoft — Dynamic-link library search order"
