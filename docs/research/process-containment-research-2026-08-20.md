# Pesquisa de contenção de processos — 2026-08-20

**Projeto:** Rovex

## Hipótese auditada

O worker de conversão criava pipes para stdout/stderr, matava o processo FFmpeg/ffprobe direto e depois fazia `join` dos leitores. A hipótese era que um descendente criado pelo backend pudesse herdar um pipe e mantê-lo aberto, fazendo o cancelamento ou timeout esperar além do processo direto.

A documentação da Microsoft confirma que um processo filho pode herdar handles do pai quando o handle é criado como herdável e o processo é criado com herança habilitada; pipes estão entre os objetos que podem ser herdados.[1] Isso não prova por si só que todo `std::process::Command` herda handles de modo inseguro, mas torna a combinação “descendente + pipe + join” uma hipótese executável, não apenas teórica.

## Reprodução

O teste `cancelamento_encerra_descendente_que_mantem_pipe_aberto` usa um backend fake Unix que inicia `sleep` em background com stderr herdado, cria um marcador de prontidão e continua executando. A versão que terminava apenas o processo direto deixava o descendente com o pipe aberto; a versão corrigida termina o grupo e retorna rapidamente. O teste mede o tempo de retorno, aguarda o marcador para evitar uma corrida de inicialização e limpa todos os artefatos temporários.

## Evidência oficial para a correção

A documentação da Microsoft define Job Objects como grupos de processos administráveis como uma unidade. Operações no job afetam os processos associados, e filhos criados por `CreateProcess` são associados ao job por padrão, salvo configurações de breakaway. A mesma documentação descreve `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, que termina os processos associados quando o último handle do job é fechado.[2]

A implementação Unix usa `CommandExt::process_group(0)` e `libc::killpg`. A implementação Windows cria e configura um Job Object, associa o processo com `AssignProcessToJobObject` e termina a árvore com `TerminateJobObject`; o handle tem fechamento RAII. Se uma etapa do Job Object falhar, o caminho retorna erro após tentar terminar o processo direto, sem alegar contenção que não foi estabelecida.

## Riscos que continuam separados

Job Objects não são sandboxing e não autenticam o backend. A Microsoft informa que dependências de uma DLL são procuradas por nome mesmo quando a DLL inicial foi carregada por caminho completo; uma pasta pesquisada sob controle de um atacante pode receber uma cópia maliciosa.[3] Também é recomendável testar nativamente a associação quando o processo pai já está em outro Job Object, processos que tentam breakaway, a pequena janela entre spawn e associação, DLLs dependentes e backends Windows que criam descendentes.

Esses pontos não foram marcados como resolvidos nesta release. TOCTOU de filesystem, ACLs, UNC/SMB, arquivos bloqueados, disco cheio, DPI, acessibilidade e execução gráfica interativa em Windows 10/11 continuam fora da prova automatizada local.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/sysinfo/handle-inheritance "Microsoft — Handle Inheritance"
[2]: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects "Microsoft — Job Objects"
[3]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order "Microsoft — Dynamic-link library search order"

## Endurecimento adicional do ciclo v0.1.26

A revisão do retorno das APIs mostrou que ignorar uma falha de `killpg` ou `TerminateJobObject` deixaria o fluxo apenas com `wait` e poderia manter o mesmo risco de descendente. O helper agora examina o resultado da terminação da árvore: quando a API falha, tenta `Child::kill` como último recurso antes de aguardar. A decisão é deliberadamente conservadora: o fallback direto não é apresentado como contenção completa, e falhas de estabelecimento do Job Object continuam fazendo a operação retornar erro.
