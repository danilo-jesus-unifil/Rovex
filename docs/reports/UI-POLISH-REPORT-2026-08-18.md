# Refinamento visual e usabilidade do Rovex

**Data:** 18 de agosto de 2026  
**Base:** Rovex v0.1.9  
**Escopo:** melhoria visual orientada por padrões de exploradores de arquivos, preservação do tema escuro e validação de regressões.

## Direção de design

A interface foi comparada com padrões documentados do File Explorer do Windows e das Human Interface Guidelines do GNOME. O modelo adotado separa navegação persistente, conteúdo principal, busca/localização e ações por tarefa. O Windows organiza o File Explorer com painel lateral, barra de comandos, endereço, busca, listagem e menu contextual enxuto; também mantém pastas conhecidas em acesso rápido e diferencia modos de visualização.[1] O GNOME recomenda que sidebars contenham locais úteis, sejam ordenadas por utilidade e tenham controles relacionados posicionados acima da lista.[2]

A aplicação das referências foi deliberadamente conservadora: nenhum callback Rust, modelo de dados, fluxo de seleção, operação de arquivo ou API pública foi alterado. O trabalho concentrou-se em hierarquia, espaçamento, estados interativos, alinhamento e uso consistente dos tokens escuros.

## Alterações aplicadas

A toolbar passou a ser composta por três painéis escuros discretos, com borda sutil, raio consistente e padding interno: abas; navegação, caminho, filtro e atualização; e ações de arquivo. Essa organização evita que ações de naturezas diferentes pareçam um único grupo e melhora a leitura por tarefa.

A listagem recebeu um cabeçalho alinhado às colunas `Tipo`, `Nome` e `Detalhes`. A sidebar passou a ter título com peso tipográfico maior e divisor sutil. As linhas mantêm a seleção azul existente, recebem hover em `surface-hover` e borda accent somente no foco de teclado. O ícone de cada arquivo continua centralizado verticalmente com `cross-axis-alignment: center`.

O Tooltip nativo do Slint permanece customizado com `surface-elevated`, `border-strong` e `text-primary`. Os tooltips continuam limitados aos controles icon-only e não substituem informações essenciais. A documentação do Slint recomenda conteúdo customizado dentro do Tooltip para esse caso.[3]

Os scripts gráficos foram atualizados para a nova geometria: as abas continuam em y=35 e a primeira linha de arquivo passou para y=246, abaixo do cabeçalho de colunas. O `build.rs` continua observando todos os módulos Slint.

## Consistência do tema escuro

Foi criado um teste visual que analisa dez screenshots de tela principal, abas, menus, diálogo e tooltips. O teste tolera texto claro e antialiasing, mas falha quando uma superfície apresenta concentração indevida de pixels próximos ao branco.

| Métrica | Resultado |
|---|---:|
| Screenshots analisadas | 10 |
| Superfícies puramente brancas (`pure_white`) | 0,0000% em todas |
| Maior proporção de pixels próximos ao branco | 0,0624% |
| Amostras estruturais claras indevidas | 0 |
| Resultado | `DARK_THEME_CHECK_OK` |

As cores estruturais observadas permaneceram dentro da paleta definida em `design_tokens.slint`: fundo `#0f141a`, superfícies escuras, controles azul-acinzentados, seleção azul accent, danger vermelho e texto primário claro apenas onde necessário para legibilidade.

## Validação técnica e funcional

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 44 aprovados, 0 falhos, 2 ignorados explicitamente |
| Clippy Linux com `-D warnings` | Aprovado |
| Check e Clippy `x86_64-pc-windows-gnu` | Aprovados |
| Build release Linux | Aprovado |
| Build release Windows GNU | Aprovado |
| Smoke GUI | Aprovado |
| Fluxo de abas | Aprovado |
| Menu contextual e menu de conversões | Aprovados |
| Conversão JPEG XL, inclusive diretórios separados | Aprovada; codec `jpegxl` confirmado por `ffprobe` |
| Scripts shell | Todos aprovados por `bash -n` |
| Consistência visual escura | Aprovada por teste automatizado |

## Conclusão

O Rovex agora apresenta uma hierarquia mais próxima de um explorador de arquivos moderno: abas e comandos agrupados, navegação lateral mais legível, listagem com cabeçalho, estados de hover/foco, operações visualmente separadas e overlays escuros consistentes. O refinamento preserva o comportamento existente e foi validado no alvo Linux e no check cruzado Windows GNU.

## Referências

[1]: https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows "Microsoft Support: File Explorer in Windows"
[2]: https://developer.gnome.org/hig/patterns/nav/sidebars.html "GNOME HIG: Sidebars"
[3]: https://docs.slint.dev/latest/docs/slint/reference/window/tooltip/ "Slint Docs: Tooltip"
