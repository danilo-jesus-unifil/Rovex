# Rovex: Relatório Final de Estabilidade

> **Documento histórico:** este relatório descreve o baseline auditado em 15 de agosto de 2026, antes da v0.1.8. Para o estado atual, consulte `README.md`, `../reference/known-issues.md` e `./ARCHITECTURE-REFACTOR-REPORT-2026-08-17.md`.

**Data da auditoria:** 15 de agosto de 2026  
**Baseline auditado:** commit `d464851`  
**Checkpoint de rollback:** `backup/before-final-stabilization-2026-08-15`  
**Estado:** correções publicadas no commit `c07cff8`; CI final `31866482594` aprovado em todos os jobs.

> **Resultado:** nenhum problema conhecido relevante ficou sem correção dentro do escopo executável desta auditoria. Permanecem limitações de plataforma e funcionalidades fora da primeira fatia; elas não são cobertura concluída.

## Escopo e método

A auditoria revisou o núcleo de filesystem, segurança de caminhos, cópia atômica, worker de operações, workers latest-only de carregamento e filtro, seleção, histórico, callbacks Slint, modal, documentação de segurança e performance. Cada achado seguiu o fluxo encontrar, reproduzir, identificar causa, corrigir, testar e procurar regressão.

A análise também verificou a ausência de processos externos de conversão e revisou os blocos `unsafe` de FFI Win32 no código próprio de produção daquele baseline. Na data da auditoria, o aplicativo não continha pesquisa global, preview, thumbnails, conversores, abas, split view, drag and drop, clipboard, instalador ou atualizador; portanto, esses itens foram tratados como **fora do escopo implementado naquela versão**, não como funcionalidades presumidas. A v0.1.8 posterior adicionou abas, menu contextual e conversores; este relatório não substitui a documentação atual.

## Problemas encontrados e corrigidos

| Área | Problema reproduzido | Causa | Correção | Evidência |
|---|---|---|---|---|
| Segurança / filesystem | O fallback de publicação podia remover um destino preexistente quando `create_new` falhava | A limpeza genérica chamava `remove_file(destination)` mesmo quando a operação nunca criou o destino | Retorno imediato no erro de `create_new`; limpeza somente após criação bem-sucedida | Teste `fallback_nao_remove_destino_preexistente_quando_create_new_falha` |
| UX / navegação | Clique simples em diretório selecionava e navegava ao mesmo tempo | `pointer-event` disparava `activate` em todo release sem modificadores | Clique simples agora seleciona; `double-clicked` abre diretórios | Smoke X11 e captura `single-click` |
| Acessibilidade / teclado | Não havia caminho explícito para Enter na linha focada nem Escape na modal | O contrato Slint expunha somente callbacks de mouse e botões | Índice de linha focada, Enter para ativar e FocusScope modal com Escape para cancelar/fechar | Smoke X11 de interação |
| Consistência de estado | Índice focado podia sobreviver a reload ou filtro e apontar para outra lista | O foco visual não era invalidado junto com seleção e snapshot | Reload/filtro agora resetam `focused-row-index` para `-1` | Compilação, testes e smoke de navegação |
| Código / eventos | O handler de teclado tinha `reject` após branches com `accept` | Retorno de evento não era mutuamente exclusivo | Branch final explícito `else { reject }` | `cargo check`, testes e smoke |
| Documentação | `SECURITY.md` e auditoria de performance descreviam estado anterior à UI de operações e ao worker único | Documentação não foi reconciliada após mudanças anteriores | Política e riscos atuais atualizados, incluindo limitações residuais | Diff documental e revisão final |

## Vulnerabilidades novas verificadas

A auditoria repetiu `cargo audit`, `cargo deny check`, `cargo tree`, inventário de dependências, busca de `unsafe`, busca de processos externos e revisão de filesystem. Não foram encontrados advisories de segurança exploráveis no estado resolvido. Permanecem visíveis os quatro avisos de manutenção transitivos documentados pela cadeia Slint: `bincode`, `paste`, `rustybuzz` e `ttf-parser`.

A revisão de código próprio cobriu traversal, componentes pai symlink, raiz, destinos existentes, nomes Unicode, temporários, publicação sem sobrescrita, cancelamento, remoção limitada, erros parciais e origem/destino equivalentes. O bug de limpeza destrutiva no fallback foi corrigido e recebeu teste de regressão. A política completa permanece em [`SECURITY.md`](../../SECURITY.md).

Não há execução de shell, PowerShell, CMD, FFmpeg, OCR, compactador ou outro processo externo no produto daquele baseline. Os blocos `unsafe` existentes ficam restritos às integrações FFI Win32 de Known Folders, com invariantes documentadas; não há `unsafe` não justificado no caminho de produção revisado.

## Filesystem e proteção contra perda de dados

A cópia continua usando buffer de 64 KiB, arquivo temporário no diretório pai, `flush`/`sync_all`, validação de tamanho e publicação sem sobrescrita por hard link ou `create_new`. O cancelamento não publica destino parcial; em movimentação entre volumes, a origem permanece quando a cópia terminou mas o cancelamento foi observado antes da remoção.

A exclusão continua deliberadamente limitada a arquivos, links e diretórios vazios. Diretórios não vazios são recusados sem recursão. A validação de caminhos relativos, raízes, componentes pai symlink e destinos existentes continua ativa.

Uma limitação residual permanece: filesystems comuns não oferecem, por esta API portátil, uma transação completa entre validação e todas as syscalls subsequentes contra qualquer alteração externa concorrente. A implementação reduz a consequência perigosa e nunca remove um destino que não tenha sido criado pelo próprio processo, mas corridas externas de permissão, rename, remoção ou substituição continuam sendo erros recuperáveis a serem tratados pelo usuário.

## Concorrência e consistência de estado

O carregamento de diretórios usa um worker único latest-only com geração atômica e encerramento cooperativo. O filtro usa worker dedicado com fila efetiva de capacidade um, snapshots compartilhados e descarte por geração. O worker de operações é separado do event loop, aceita uma operação por vez, publica progresso limitado por percentual, suporta cancelamento cooperativo e agenda reload da pasta após o resultado.

O stress X11 criou 12 pastas com 80 arquivos cada, enviou 12 navegações rápidas e consultas de filtro sobre o diretório final. A última navegação chegou a `folder-12`; a consulta `file-12-12` exibiu somente `file-12-12.txt`. O encerramento do processo foi verificado por término controlado; `WM_DELETE_WINDOW` isolado não foi usado como prova de fechamento no Xvfb sem window manager.

Não foram observados deadlocks, filas infinitas, threads criadas por tecla ou workers persistentes após término controlado. A materialização completa de uma pasta em snapshot continua sendo o principal risco de memória em diretórios muito grandes, conforme [`../audits/performance-audit-2026-08-15.md`](../audits/performance-audit-2026-08-15.md).

## UI, UX e comportamento funcional

A interação da lista agora segue o comportamento familiar de exploradores: clique simples seleciona, double-click abre diretórios e Enter ativa a linha focada. A modal bloqueia a UI subjacente; Escape cancela uma operação em andamento ou fecha uma confirmação/resultado quando não há operação ativa. A seleção é limpa ao filtrar ou recarregar, evitando agir sobre itens invisíveis ou sobre um índice obsoleto.

A suíte visual mostrou toolbar, sidebar, lista, estado vazio, seleção e filtro alinhados ao Design System existente. Não foram adicionados efeitos contínuos, animações, thumbnails ou dependências pesadas. As operações de copiar, mover, renomear e excluir continuam centralizadas no worker e na camada de operações, sem implementações independentes por caminho visual.

## Testes realizados

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo check --all-targets --no-default-features` | Aprovado |
| `cargo test --all-targets --all-features` | 32 aprovados, 0 falhas, 1 ignorado |
| `cargo test --all-targets --no-default-features` | Aprovado |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo doc --all-features --no-deps` | Aprovado |
| `cargo audit` | Aprovado, sem vulnerabilidade explorável; avisos transitivos mantidos visíveis |
| `cargo deny check` | Aprovado: advisories, bans, licenças e fontes |
| Release Linux | Aprovado; `16.333.200` bytes |
| Release Windows GNU | Aprovado; `12.679.680` bytes |
| Smoke de operações | `copy=ok`, `rename=ok`, `delete=ok`, filesystem verificado |
| Smoke de interação | Clique simples, double-click, Enter e Escape aprovados |
| Stress latest-only | 12 navegações rápidas, filtro positivo e limpeza controlada aprovados |
| Revisão visual | Capturas finais revisadas em 1100×720 |

## Regressões e correções

A regressão mais grave foi a possibilidade de apagar um destino preexistente em uma falha rara do fallback de publicação; ela foi corrigida antes da suíte final. A regressão de UX era a abertura por clique simples, que foi alterada para double-click e Enter sem remover a capacidade de seleção múltipla. A regressão documental foi corrigida para que segurança e performance descrevam o estado efetivamente publicado.

Nenhuma regressão foi encontrada na suíte final depois das correções. O novo teste do núcleo elevou a cobertura executada para 32 testes aprovados, com um benchmark manual intencionalmente ignorado.

## Performance e consumo

O caminho de cópia usa streaming em blocos e não lê o arquivo integral para a RAM. O filtro e o carregador são latest-only. A auditoria histórica mediu 100.000 entradas e documentou que a lista visual usa `ListView`, embora os metadados completos ainda sejam materializados em memória. O stress atual confirmou responsividade e descarte de resultados obsoletos, mas não substitui benchmark prolongado de 100.000+ entradas em hardware Windows modesto. A CI final `31866482594` aprovou a compilação release Linux e Windows, além do cross-build GNU.

Não foram identificados loops de polling, timers, thumbnails, previews, hashes ou conversores no startup. Não foi declarado ganho artificial de CPU/RAM; as métricas de baseline permanecem no relatório de performance para comparação futura.

## Compatibilidade Windows 10/11

O projeto compila release x86-64 com target `x86_64-pc-windows-gnu`, e o CI deve validar runner Windows além do cross-build GNU. A escolha de Windows 10 22H2/build 19045 continua uma política de teste, não uma execução nativa realizada nesta sessão.

Ainda não foram comprovados nativamente nesta sessão: Windows 10 22H2, Windows 11 em versão identificada, DPI 100/125/150/200%, múltiplos monitores, alto contraste, leitor de tela nativo, paths longos, UNC/SMB, NTFS/exFAT/FAT32 em hardware real, USB desconectado, reparse points/junctions, arquivo bloqueado, permissões Windows, manifesto PE, instalador, desinstalador, assinatura ou atualização. Esses itens permanecem pendências honestas em [`COMPATIBILITY.md`](../../COMPATIBILITY.md).

## Problemas ainda conhecidos

A primeira fatia não oferece abas, split view, pesquisa global, preview, thumbnails, drag and drop, clipboard, menu contextual, favoritos persistentes, tema configurável, conversores, OCR, instalador, atualização automática ou undo/redo. A exclusão recursiva não existe por decisão de segurança. A UI ainda materializa metadados completos de uma pasta antes de publicar o snapshot. Filesystems externos podem mudar entre validação e execução; nesses casos o resultado esperado é erro controlado, não garantia transacional universal.

## Referências

[1]: ../../SECURITY.md "Política de segurança do Rovex"
[2]: ../audits/performance-audit-2026-08-15.md "Auditoria de performance do Rovex"
[3]: ../../COMPATIBILITY.md "Matriz de compatibilidade do Rovex"
[4]: ../reference/../reference/DEPENDENCIES.md "Inventário de dependências do Rovex"
[5]: ../reference/known-issues.md "Limitações conhecidas do Rovex"
