# Validação visual — 14/08/2026

O smoke release em Xvfb abriu a janela Rovex em 1100×720, manteve os botões de navegação, a barra de endereço e o campo de filtro visíveis, e filtrou `/tmp` para cinco entradas correspondentes a `cargo` sem travar a UI. O screenshot está em `/tmp/rovex-filter-smoke.png` durante a sessão local.

O smoke release de seleção criou quatro arquivos temporários, executou clique normal, Ctrl-clique, Shift-clique e Ctrl+A na lista, e capturou `/tmp/rovex-selection-smoke.png`. O screenshot mostra as quatro linhas com fundo azul de seleção e o status `4 itens selecionados`, confirmando o caminho real de interação da UI e não apenas o teste unitário.
O smoke de histórico abriu uma raiz temporária contendo `subdir` e `root.txt`, navegou para `subdir`, voltou para a raiz e avançou novamente. `/tmp/rovex-history-back.png` mostra a raiz e o botão Avançar habilitado; `/tmp/rovex-history-forward.png` mostra a subpasta e o botão Voltar habilitado, com `child.txt` listado. Isso valida as transições reais do histórico, não apenas a estrutura unitária.
