# Plano de refatoração visual do Rovex

## Direção

A interface manterá o tema escuro azul-marinho existente, mas passará a usar uma hierarquia de superfícies mais clara, controles com estados consistentes, raios de 8-12 px, espaçamento baseado em múltiplos de 4 px e uma barra superior dividida semanticamente entre navegação, localização e ferramentas. A prioridade é preservar as callbacks e os workers atuais.

## Alterações visuais

| Área | Decisão |
|---|---|
| Tokens | Separar superfícies da janela, painel, lista, hover, seleção, diálogo e perigo; criar raios small/medium/large, alturas de controle e espaçamentos de toolbar. |
| Botões | Criar variantes de ação, ícone, secundário e perigo dentro do mesmo componente; manter acessibilidade e teclado. |
| Atualizar | Trocar o texto por símbolo de reload, com `accessible-label` e tooltip visual ao hover. O callback `refresh-requested` permanece inalterado. |
| Navegação | Aumentar separação entre voltar, avançar e subir; agrupar caminho e filtro; manter histórico atual. |
| Lista | Substituir `[DIR]`, `[FILE]`, `[LINK]` e `[OTHER]` por glifos curtos com label acessível e manter detalhes/seleção. |
| Menu contextual | Manter todas as ações, mas usar cabeçalho, grupos, separador e variante de perigo; explicar conversões desabilitadas. |
| Diálogos | Hierarquia de título, mensagem, progresso, confirmação e cancelamento; raios e espaçamento compartilhados. |
| Ícone | Usar o conceito `assets/icon-concepts/rovex-icon-b.png`, convertido para PNG/ICO, com símbolo de pasta e navegação em paleta azul do Rovex. |
| Issue #1 | Implementar uma faixa de abas de navegação com uma aba inicial real e nova aba independente, sem remover voltar/avançar. Cada aba terá caminho atual e histórico próprio; o carregador existente continuará recebendo somente o caminho da aba ativa. |

## Critérios de aceitação

A janela deve continuar legível em 720×480 e 1200×800, sem controles cortados. O refresh precisa ser reconhecível por tooltip/label, todas as ações existentes devem continuar disponíveis, o menu contextual deve manter as quatro conversões, e o issue de abas deve ter um fluxo real verificável. A validação será feita com screenshots antes/depois, navegação, seleção, menu contextual, diálogo de operação, conversão e teclado.
