# Pesquisa de refinamento visual do Rovex

**Data:** 17 de agosto de 2026

## Fontes e padrões observados

A documentação oficial do File Explorer do Windows 10/11 descreve um explorador centrado em um painel lateral de navegação, uma barra de comandos com ações frequentes, uma barra de endereço, busca, listagem de itens e menus contextuais enxutos. O Windows coloca ações comuns de arquivos no topo do menu contextual e oferece diferentes modos de visualização, incluindo lista, detalhes, ícones e modo compacto.[1]

A mesma documentação descreve as pastas conhecidas — Desktop, Documents, Downloads, Pictures, Music e Videos — como locais de acesso rápido no painel lateral. O padrão é manter esses destinos persistentes, destacar o local atual e reservar a área principal para os itens do diretório.[1]

As Human Interface Guidelines do GNOME organizam decisões de UX em princípios, diretrizes e padrões de containers, navegação, feedback e controles. Essa separação reforça que a UI deve ter uma hierarquia clara: navegação persistente de um lado, conteúdo principal no centro, ações agrupadas por tarefa e feedback em uma região consistente.[2]

## Critérios aplicados ao Rovex

O refinamento deve manter a estrutura de explorador com três regiões: navegação lateral persistente, conteúdo principal com cabeçalho de localização e listagem, e feedback/status em uma barra inferior. A toolbar deve agrupar navegação, localização e busca sem misturar ações destrutivas ou de conversão. As operações de arquivo permanecem em uma faixa própria e seus estados desabilitados devem usar contraste reduzido, sem desaparecer.

A hierarquia de cor deve usar exclusivamente os tokens do tema: fundo de janela, superfícies elevadas, superfície selecionada, controles, borda, texto primário, texto secundário, texto muted, accent e danger. O refinamento não deve introduzir branco puro em componentes de produção, exceto texto primário deliberado ou contraste de acessibilidade. Tooltips devem ser reservados a controles icon-only e devem continuar usando o painel escuro customizado já validado.

As melhorias devem evitar mudanças de API Rust, callbacks ou fluxos de operação. O trabalho será limitado a composição, espaçamento, agrupamento visual, estados de hover/foco/seleção, alinhamento e legibilidade.

## Referências

[1]: https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows "Microsoft Support — File Explorer in Windows"
[2]: https://developer.gnome.org/hig/ "GNOME Human Interface Guidelines"

As diretrizes de navegação do GNOME tratam sidebars como painéis verticais para uma lista de locais e recomendam ordenar os destinos pela utilidade para o usuário; controles que afetam a lista devem ficar acima dela. O mesmo conjunto separa padrões de browsing, tabs, sidebars e search, reforçando que cada grupo deve ter uma função reconhecível.[3] Isso orienta o Rovex a manter `Locais` como navegação persistente, melhorar a ordem/seleção dos destinos e evitar misturar ações de arquivo dentro do painel.

[3]: https://developer.gnome.org/hig/patterns/nav/sidebars.html "GNOME HIG — Sidebars"

## Primeira iteração visual executada

A listagem recebeu um cabeçalho discreto com as colunas `Tipo`, `Nome` e `Detalhes`, alinhado às mesmas colunas dos itens. A sidebar recebeu título com hierarquia tipográfica e divisor sutil. A inspeção do release mostrou melhor escaneabilidade e separação entre navegação e conteúdo, sem introduzir superfícies claras ou alterar os callbacks.

As linhas de arquivo também passaram a ter estado de hover e foco de teclado explícito em `surface-hover`, com borda accent apenas no foco, mantendo a seleção em `surface-selected`.

A inspeção do release após a reorganização mostrou três painéis escuros coerentes para abas, navegação/busca e operações. A listagem ganhou cabeçalho de colunas alinhado aos dados, e a sidebar ganhou título e divisor. O menu contextual regenerado também usa superfície escura, botões agrupados e estado danger vermelho sem introduzir painel branco. A seleção de `.font-unix` no fixture `/tmp` deixou claro que os estados disabled das conversões permanecem legíveis e visualmente distintos.

A revisão final das screenshots confirmou que a tela principal mantém fundo `#0f141a`, superfícies entre `#171d24` e `#24303d`, texto claro controlado por tokens e nenhum painel branco indevido. O diálogo de conversão usa `surface-dialog`, overlay escuro, botão primary azul e botão secundário escuro, preservando contraste e hierarquia. A métrica automatizada encontrou `pure_white=0.0000%` em todas as 10 screenshots e `near_white` abaixo de 0,063%, compatível com antialiasing e texto claro.
