# Contrato de busca recursiva — 18/08/2026

## Escopo

A busca do Rovex será uma busca por nome em uma árvore de diretórios local, sem abrir o conteúdo dos arquivos e sem executar qualquer entrada. O filtro atual, explicitamente rotulado `Filtrar nesta pasta`, continuará local e instantâneo sobre o snapshot carregado. A busca recursiva será uma operação distinta, iniciada por ação explícita do usuário, para não transformar cada edição do campo existente em uma varredura potencialmente ilimitada.

## Contrato do engine

O engine receberá uma raiz absoluta, uma consulta Unicode não vazia, `ListingOptions`, um limite máximo de resultados e um `Arc<AtomicBool>` de cancelamento. A consulta será comparada de modo case-insensitive por `to_lowercase()` contra o nome da entrada; não haverá leitura de conteúdo. O traversal será iterativo, com pilha de diretórios, e ordenará cada diretório pelo nome antes de expandi-lo para manter ordem determinística no host.

O engine usará `symlink_metadata` e nunca descerá em entradas simbólicas. No Windows, entradas marcadas como reparse point também serão tratadas como não-descendíveis; o caminho será no máximo reportado como item, nunca seguido. Falhas de `read_dir` ou de metadata em uma subárvore serão contabilizadas como ignoradas e não interromperão a busca inteira. A raiz inválida produzirá erro estruturado. A busca não seguirá links, junctions, componentes pai simbólicos, raízes alternativas ou caminhos relativos fornecidos pelo usuário.

## Limites e cancelamento

Os limites obrigatórios da primeira versão são `max_results`, `max_visited_directories` e `max_visited_entries`. Quando um limite for atingido, o engine interromperá de forma controlada e emitirá um estado truncado; não declarará que todos os resultados foram encontrados. O cancelamento será consultado antes de abrir cada diretório e a cada entrada visitada. O estado final distinguirá `Completed`, `Cancelled`, `Limited` e `Failed`, com contagens de encontrados, visitados e ignorados.

Os resultados serão emitidos em lotes pequenos pelo worker, após cada lote ou janela de tempo, sem tocar diretamente o modelo Slint a partir da thread de filesystem. Cada solicitação terá uma geração monotônica; o UI só aceitará atualização da geração ativa, descartando resultados obsoletos quando o usuário iniciar outra busca, navegar, atualizar ou cancelar.

## Integração e segurança

A UI terá ação explícita para iniciar a busca recursiva a partir da pasta atual e uma ação de cancelamento. Uma busca ativa não poderá iniciar outra operação concorrente usando o mesmo scheduler; uma nova solicitação cancelará a anterior e substituirá a geração. O resultado reutilizará `LoadedRow` somente depois de converter cada `DirectoryEntry` com chaves estáveis baseadas no caminho e um índice local do batch. Nenhum resultado será tratado como sucesso antes do estado final do engine.

| Critério | Evidência exigida |
|---|---|
| Não bloquear UI | worker dedicado e atualizações incrementais por `upgrade_in_event_loop` |
| Cancelamento | teste que interrompe árvore grande e verifica estado `Cancelled` |
| Segurança de links | teste que cria symlink para fora e confirma que o alvo não é visitado |
| Limites | testes para resultados, diretórios e entradas visitados |
| Erros parciais | teste com subdiretório inacessível/ausente quando o host permitir; estado contabiliza ignorados |
| Geração | teste de duas buscas em sequência em que a primeira não publica resultados obsoletos |
| Windows | cross-check GNU; reparse point e execução Explorer nativa continuam pendentes |

A busca por índice do Windows não será adicionada nesta primeira versão: ela exigiria adapter, permissões, diferenças de disponibilidade e uma estratégia de fallback. O traversal local fornece comportamento verificável e compatível com o modo sem privilégios; indexação pode ser estudada como lote posterior.
