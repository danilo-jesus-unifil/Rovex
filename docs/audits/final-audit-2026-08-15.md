# Auditoria final de segurança, bugs, UX e UI: Rovex

## Estado da auditoria

Esta auditoria foi iniciada a partir do prompt final anexado em 15 de agosto de 2026. O checkpoint remoto `backup/before-final-stabilization-2026-08-15` protege o estado modernizado publicado antes das correções desta rodada. O objetivo é registrar somente problemas reproduzidos, causas verificadas, correções implementadas e limitações que permanecem explícitas.

## Achados já reproduzidos e corrigidos

| Área | Problema reproduzido | Causa | Correção | Regressão |
|---|---|---|---|---|
| Cópia/concorrência | Um destino poderia ser criado entre a validação inicial e o `rename` do temporário, permitindo publicação destrutiva em uma janela de corrida | `rename` substitui o destino em plataformas Unix e pode ter semântica de substituição distinta entre sistemas | Publicação agora usa `hard_link` para criar o destino somente se ele ainda não existir; o temporário só é removido depois da publicação | Teste `publicacao_atomica_nao_sobrescreve_destino_criado_depois_da_validacao` aprovado |
| Nomes Unicode inválidos | Dois nomes diferentes que viram o mesmo caractere de substituição em `to_string_lossy` poderiam compartilhar identidade visual/chave; ativação reconstruída pelo nome poderia perder o alvo original | A UI recebia somente texto lossless e a chave usava apenas a representação lossy do caminho | Chave discriminada por índice e `PathBuf` original preservado no snapshot; ativação resolve a linha por chave, nunca por nome convertido | Teste Unix `preserva_caminhos_de_nomes_invalidos_sem_colidir_chaves` aprovado |
| Path traversal/ambiguidade | Operações aceitavam caminhos relativos e podiam atravessar componentes pai symlink | Validação canonicalizava o pai, mas não recusava explicitamente caminho relativo nem symlink em componentes do pai | Destinos relativos agora são recusados; cada componente pai é validado com `symlink_metadata`, e symlinks são rejeitados | Testes de caminho relativo e componente symlink aprovados |
| Mensagens de erro | Erros exibiam `ErrorKind` e códigos OS diretamente ao usuário | `Display` formatava o diagnóstico técnico como mensagem principal | Mensagens humanizadas preservam detalhes estruturados internamente | Testes de microcopy para acesso negado aprovados |
| Estado vazio | Pasta sem entradas aparecia somente como `0 itens`, sem orientação; filtro sem resultados não tinha estado visual dedicado | A UI não tinha propriedade de empty state | Added distinct messages `Esta pasta está vazia.` and `Nenhum item corresponde ao filtro.` | Teste unitário e smoke visual aprovados |
| Fallback de publicação | Um destino preexistente podia ser removido se `create_new` falhasse antes do bloco de limpeza | A limpeza genérica não distinguia arquivo criado pelo processo de arquivo preexistente | Retorno imediato no erro de criação e cleanup somente após `create_new` bem-sucedido | `fallback_nao_remove_destino_preexistente_quando_create_new_falha` |
| Interação da lista | Clique simples também ativava diretórios | O callback de pointer chamava `activate` em todo release | Seleção ficou separada de double-click/Enter | Smoke X11 de interação aprovado |

## Evidência visual

O smoke `/tmp/rovex-empty-state-smoke.png` abriu uma pasta temporária real e mostrou a mensagem `Esta pasta está vazia.` centralizada no painel, com status `0 itens`, sidebar, barra de endereço, filtro e controles de navegação intactos.
O smoke `/tmp/rovex-sidebar-keyboard-smoke.png` confirmou navegação sem mouse na ação final: após foco inicial, `Down` e `Enter` carregaram `/home/ubuntu/Downloads`. A linha `Downloads` ficou destacada, o caminho exibido corresponde ao destino real e a mensagem de pasta vazia permaneceu consistente.
O smoke `/tmp/rovex-small-window-smoke.png` em 720×480 não mostrou sobreposição: toolbar, caminho, filtro, Atualizar, sidebar, lista e status permanecem visíveis. A lista usa elide nos nomes e o painel conserva scroll vertical, atendendo ao limite mínimo atual da janela.
## Achados adicionais corrigidos e verificados

A primeira implementação do empty state revelou uma regressão durante a própria auditoria: um diretório inexistente produzia zero linhas e poderia ser descrito como vazio. A causa foi a ausência de uma marca explícita de erro em `LoadedDirectory`. O modelo agora carrega `is_error`, mantém somente o status de erro e não mostra a mensagem de pasta vazia quando a listagem falha; a regressão `erro_de_diretorio_inexistente_vira_status_controlado` cobre essa invariável.

O caminho de publicação sem sobrescrita usa `hard_link` quando disponível e fallback `OpenOptions::create_new` para volumes que não suportam hard links, como alguns cenários de rede. O fallback valida tamanho, sincroniza o destino e remove somente o arquivo parcial que ele próprio criou em caso de falha; destinos preexistentes permanecem intactos mesmo quando `create_new` falha antes da entrada no bloco de cleanup.

A sidebar recebeu `FocusScope` com foco visível, setas Up/Down e Enter/Space. O smoke real confirmou `Down` + `Enter` até `/home/ubuntu/Downloads`. Em 720×480, um smoke adicional confirmou que toolbar, filtro, sidebar, lista, estado vazio e status continuam sem sobreposição.
A inspeção final confirmou que a seleção múltipla continua exibindo quatro linhas selecionadas e status `4 itens selecionados` após todas as correções. A janela mínima de 720×480 continua sem sobreposição, com nomes truncados quando necessário e status visível. A bateria final de smoke release passou para filtro, seleção, histórico, sidebar, estado vazio, sidebar por teclado e janela mínima.
## Dependências e limites de verificação

`cargo audit` terminou com código 0 e `cargo deny check` aprovou advisories, bans, licenças e fontes; permanecem quatro avisos transitivos de manutenção do toolkit Slint (`bincode`, `paste`, `rustybuzz` e `ttf-parser`), sem vulnerabilidade reportada nessa execução. `cargo outdated` não estava instalado no ambiente e não foi tratado como disponível; a ausência foi registrada em vez de simular uma análise. A árvore possui duplicações transitivas esperadas do toolkit, sem nova dependência direta adicionada nesta auditoria.

A CI histórica do commit `a9e32e8` foi aprovada na execução `31857990905`. A ponta desta rodada, commit `c07cff8`, foi aprovada na execução `31866482594`, com jobs Linux, Windows, auditoria de dependências e cross-build Windows GNU. A auditoria não afirma ausência absoluta de bugs: afirma que nenhum problema conhecido relevante permaneceu dentro do escopo testado, com execução nativa Windows 10/11, paths longos, reparse points, UNC/SMB, DPI nativo, instalador e operações visuais ainda pendentes.
