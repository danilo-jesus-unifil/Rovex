# Auditoria de performance e design: 15/08/2026

## Estado observado

O Rovex usa Slint 1.17.1 com backend Winit, renderer software, acessibilidade e apenas a dependência direta do toolkit. O perfil release já usa `codegen-units = 1`, LTO thin, strip de símbolos e `panic = abort`. A UI usa `ListView`, portanto a representação visual não cria 100.000 linhas simultaneamente; contudo, o carregador materializa todos os metadados da pasta em `Vec<LoadedRow>` antes de publicar um snapshot.

## Desperdícios potenciais identificados

| Área | Observação | Prioridade | Decisão inicial |
|---|---|---:|---|
| Carregamento | O estado atual usa um worker único latest-only persistente, com fila efetiva de capacidade 1, geração e encerramento cooperativo; o risco restante é materializar a pasta inteira antes da publicação. | Média | Manter a arquitetura; avaliar streaming/paginação somente com demanda e benchmark de pastas enormes. |
| Filtro | Existe um worker dedicado persistente e uma fila de capacidade efetiva 1, o que evita threads por tecla e filas infinitas. | Baixa | Manter; revisar encerramento lifecycle e custo de normalização após medir. |
| Filtro | `filter_rows` aloca `to_lowercase()` para a consulta e para cada nome a cada consulta; também clona as linhas filtradas. | Média | Medir em 100.000 itens; considerar cache de chave normalizada ou comparação case-insensitive com menos alocações, sem duplicar memória sem evidência. |
| Estado | `SharedRows` armazena uma única `Arc<[LoadedRow]>`; o modelo Slint possui outra representação das linhas visíveis. | Média | Manter por segurança do event loop; evitar cópias adicionais e avaliar atualização incremental somente se necessário. |
| Seleção | Chaves são `String` derivadas de `to_string_lossy()` e a seleção usa `BTreeSet<String>`. | Baixa | Não alterar sem caso reproduzível; preserva compatibilidade com nomes comuns e mantém ordenação determinística. |
| Locais | `default_locations` faz poucas chamadas `is_dir` para HOME, pastas conhecidas, pasta atual e raiz Unix. | Baixa | Manter: conjunto constante, barato, sem scan global ou análise de espaço. |
| UI visual | Cores, raios, espaçamentos e alturas estão repetidos diretamente em `main.slint`; botões, campos, sidebar e lista não compartilham tokens. | Média | Criar tokens Slint locais e componentes reutilizáveis leves; não introduzir animações, blur ou dependências. |
| Background | Não há polling, timers, thumbnails, previews, pesquisa global, hash ou conversores ativos no startup. | Correto | Preservar conforme RUSTORA. |

## Auditoria de segurança/performance

Não há `fs::read`, `read_to_string`, previews ou carregamento de arquivos grandes no caminho da UI. O filesystem usa `symlink_metadata` e lista somente uma pasta. O filtro não faz pesquisa recursiva. O risco principal para a responsividade é a materialização completa de diretórios muito grandes e a duplicação temporária de snapshots/modelos, não a criação repetida de workers; a navegação rápida agora reutiliza um worker único.

A próxima fase deve medir tempo de listagem, primeira publicação, filtro de 100.000 linhas e RSS/CPU em idle. O smoke de estabilização já executou 12 navegações rápidas seguidas de consultas latest-only e confirmou que `folder-12` e `file-12-12.txt` venceram resultados obsoletos. Nenhuma otimização deve ser declarada como ganho antes da comparação antes/depois.
## Linha de base medida

A medição executada em 15/08/2026 criou 100.000 arquivos temporários. O modo CLI retornou todas as 100.000 entradas, terminou em `0,332518 s` e apresentou pico de RSS de `30.356 KiB` no processo filho. A janela release apareceu em `117 ms` sob Xvfb; após um segundo em uma pasta de 100.000 arquivos, o processo apresentou aproximadamente `82.980 KiB` de RSS, `177.620 KiB` de VSZ e `36%` de CPU na amostragem pontual. A amostra de CPU não deve ser tratada como consumo idle estável, pois inclui o carregamento inicial e a amostragem ocorreu durante a listagem.

A linha de base confirma que o renderer usa `ListView` sem 100.000 componentes visuais ativos, mas também confirma que o carregamento da pasta ainda materializa a listagem completa e faz trabalho de filesystem na inicialização da pasta. O próximo passo deve medir navegação rápida e filtro antes/depois de qualquer mudança no worker de carregamento.
O benchmark manual release `benchmark_filtro_100k` processou 100.000 linhas com a consulta `99999` em `9,732455 ms` e encontrou um resultado. Essa é a referência de CPU do filtro atual no runner; a otimização deve ser justificada contra esse valor e não apenas por aparência do código.
## Medição após o worker único de carregamento

Após substituir a criação de uma thread por navegação por um worker único latest-only, a repetição com 100.000 arquivos retornou todas as entradas em `0,348735 s`, com pico de RSS CLI de `30.104 KiB`. A linha de base havia sido `0,332518 s` e `30.356 KiB`; portanto, nesta amostra o RSS caiu `252 KiB`, mas o tempo CLI ficou aproximadamente `4,9%` acima. A abertura da janela foi medida em `344 ms` nesta execução contra `117 ms` na primeira amostra, diferença que exige medições repetidas antes de qualquer conclusão; a troca não deve ser apresentada como ganho de startup. O objetivo comprovado da alteração é reduzir a multiplicação de threads durante navegação rápida e garantir encerramento cooperativo, não acelerar uma única listagem.
## Auditoria visual após tokens

O screenshot `/tmp/rovex-sidebar-smoke.png` mostra `Início` destacado como local atual com o token de seleção, texto em accent e superfícies discretas. O screenshot `/tmp/rovex-selection-smoke.png` mostra `Pasta atual` destacada na sidebar e as quatro linhas do painel principal selecionadas com o mesmo azul claro, sem brilho, sombra ou animação contínua. A hierarquia permanece toolbar → sidebar/content → lista → status, com alinhamento consistente e sem novas camadas pesadas.
A inspeção visual final após a última normalização confirmou novamente a sidebar com o local atual destacado e o filtro `cargo` com cinco resultados. A UI manteve toolbar, sidebar, lista e status alinhados, sem alterações de comportamento ou efeitos contínuos.
