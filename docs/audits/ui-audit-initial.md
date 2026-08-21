# Auditoria inicial da interface Rovex

## Git e issues

## main...origin/main

Issues:
1	OPEN	ideia de ui para o gerenciador de arquivos		2026-08-17T19:42:30Z

## Pontos observados no Slint

- DesignTokens já existe, mas usa raio único de 4px, refresh textual e componentes sem variantes semânticas.
- Toolbar mistura controles de navegação, caminho, filtro e refresh numa única linha, com refresh textual de 92px.
- Menu contextual e diálogo repetem botões sem hierarquia visual primária/secundária.
- Lista usa marcadores textuais de tipo em vez de um sistema de ícones consistente.
- O código preserva callbacks para todas as operações e conversões; a refatoração deve manter essas interfaces.

## Referências visuais consultadas

A documentação da Microsoft mostra o padrão do File Explorer do Windows 11 com ações frequentes representadas por ícones no topo do menu contextual, enquanto os nomes aparecem por tooltip ao passar o cursor. Fonte: [Microsoft Support: File Explorer in Windows](https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows).

A página oficial do GNOME Files destaca uma organização simples de gerenciador de arquivos, com sidebar, navegação, seleção múltipla, busca e modos de visualização distintos. Fonte: [GNOME Apps: Files](https://apps.gnome.org/Nautilus/).

Implicações para o Rovex: manter o tema escuro e a identidade azul atual, substituir o refresh textual por um botão iconográfico com label acessível, criar variantes visuais de botão sem duplicar estilos, preservar a sidebar e a lista detalhada, melhorar hierarquia, raios e espaçamento e avaliar uma camada de abas para atender o issue #1 sem remover o histórico existente.

## Capturas visuais de baseline

A captura `artifacts/rovex-dark-theme.png` mostra uma janela funcional, porém com excesso de retângulos da mesma força visual, controles pequenos para a dimensão da janela, toolbar sem agrupamento semântico e refresh textual competindo com o campo de caminho. A lista é legível, mas os tipos aparecem como texto `[DIR]`, o que reduz a leitura imediata.

A captura `artifacts/rovex-context-menu.png` mostra que o menu contextual cobre a barra lateral e parte da lista, repete quatro botões com o mesmo peso, usa uma explicação de FFmpeg muito discreta e deixa as quatro conversões desabilitadas sem uma explicação contextual suficientemente visível. O agrupamento por separador existe, mas a hierarquia primária/secundária ainda é fraca.

## Direção visual aprovada para implementação

A evolução deve manter a paleta escura azulada, mas aumentar a diferenciação entre janela, superfície, painel e item selecionado; usar raios maiores e consistentes; ampliar espaçamentos da toolbar; trocar refresh por ícone com tooltip/label acessível; criar botão iconográfico reutilizável; usar símbolos de tipo mais compactos na lista; diferenciar ações perigosas; melhorar cabeçalho, estados vazios e diálogo; e preservar todos os callbacks atuais. O issue de abas será tratado como uma extensão de navegação controlada, sem remover o histórico de voltar/avançar.

## Segunda rodada de visualização

A captura atualizada `artifacts/rovex-dark-theme.png` confirma melhora significativa: o cabeçalho introduz marca e estado, os controles têm mais respiro, o refresh aparece como símbolo, os botões têm cantos e contraste mais consistentes e a lista agora usa um marcador visual para pastas. A densidade ainda pode ser reduzida com a faixa de abas e uma melhor diferenciação entre ícone, tipo e nome.

A captura `artifacts/rovex-context-menu.png` não exibiu o menu contextual: o script existente ainda usa a coordenada vertical antiga, que deixou de atingir uma linha de arquivo depois do novo cabeçalho. Isso é uma regressão do teste, não da UI; o script será atualizado para localizar uma linha pela geometria atual antes da validação final.

## Regressão encontrada no ciclo de testes

Depois de corrigir as coordenadas, o menu contextual abriu, mas as quatro conversões permaneceram desabilitadas para uma imagem válida. A causa foi identificada no callback da UI: a refatoração trocou o texto interno de tipo de `[FILE]` para `Arquivo`, enquanto o código ainda verificava literalmente `[FILE]` para determinar `is_regular_file`. A correção foi feita usando a representação semântica atual, seguida de `cargo check`, build release e nova conversão real bem-sucedida.

A captura `artifacts/rovex-context-menu.png` da pasta continua corretamente com conversões desabilitadas, porque o item selecionado é um diretório. A captura `artifacts/rovex-conversion-menu.png` foi regenerada com o fixture de imagem e confirmou visualmente que JPEG XL e PNG ficam habilitados, enquanto as conversões de áudio permanecem desabilitadas para uma imagem.

## Segunda regressão do smoke test após abas

O teste de JPEG XL chegou corretamente ao diálogo real de confirmação, como mostra `artifacts/rovex-jxl-confirm.png`. A saída não foi publicada porque a coordenada antiga do script (`x=540`) agora aciona o botão Voltar; o novo botão Confirmar está aproximadamente em `x=220, y=488`. O script foi corrigido e o ciclo seguinte passou.

## Evidência visual final

A captura `artifacts/rovex-tabs-two.png` confirma duas abas reais, com uma ativa, botão de fechamento independente e botão de nova aba, sem comprimir ou cortar a toolbar. O smoke test também alternou para a primeira aba e fechou a segunda mantendo o processo vivo.

A captura final `artifacts/rovex-conversion-menu.png` confirma a hierarquia visual revisada: ações de arquivo no topo, exclusão em vermelho, JPEG XL e PNG habilitados para uma imagem, e conversões de áudio corretamente desabilitadas para o tipo selecionado. O fluxo JPEG XL normal e o fluxo com binário/imagem em diretórios separados produziram e validaram uma saída real `jpegxl`.
