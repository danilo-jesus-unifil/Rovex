# Pesquisa de compatibilidade e composição de UI — 17 de agosto de 2026

## Fontes consultadas

A documentação oficial do backend Winit do Slint informa que ele usa winit e suporta Windows, macOS e Linux com Wayland ou X11. A mesma página documenta o renderer software, que não exige GPU, e explica que no Linux o suporte X11/Wayland é escolhido por feature de compilação e exige bibliotecas de runtime correspondentes.[1]

A documentação oficial de `PopupWindow` descreve esse componente como o mecanismo para mostrar um popup como tooltip ou menu suspenso. O componente tem ciclo de vida explícito com `show()`/`close()` e políticas de fechamento por clique ou Escape.[2]

## Implicações para o Rovex

O `Cargo.toml` do Rovex restringe `backend-winit-x11` ao alvo Linux e usa `backend-winit` no alvo Windows. Essa diferença é uma seleção correta de backend por plataforma, não uma arquitetura exclusiva de Linux. O código de descoberta, filesystem, estado, operações e handlers usa `cfg(windows)` somente para APIs Windows específicas, como Registro, Known Folders e `SHGetKnownFolderPath`; o restante é compartilhado.

A regressão visual observada não é causada pelo backend Linux. O tooltip implementado no controle compartilhado está sendo desenhado dentro da árvore da janela, e a ordem de composição faz com que ele possa ficar atrás de irmãos posteriores, como toolbar, abas ou campos. Para conteúdo que precisa escapar da ordem normal de desenho, `PopupWindow` é a solução nativa indicada pela documentação; para tooltip simples, a alternativa de menor risco é renderizar a ajuda dentro do próprio controle, acima de seus filhos, sem atravessar a árvore de componentes.

## Hipótese de causa a confirmar no código

A causa provável deve ser procurada em `ui/components.slint`: o componente de botão provavelmente declara o texto de tooltip como um filho comum ou como um overlay local. Será necessário verificar se a posição do tooltip fica limitada ao stacking order do componente pai e se o campo de texto do tooltip está depois/antes dos demais elementos na árvore. A correção deve preservar a acessibilidade e evitar introduzir uma janela popup para cada hover sem necessidade.

[1]: https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_winit/ "Slint Docs — Winit Backend"
[2]: https://docs.slint.dev/latest/docs/slint/reference/window/popupwindow/ "Slint Docs — PopupWindow"

A referência correta da API é `/reference/window/tooltip/`, não `/reference/tooltip/`. A documentação de Tooltip afirma que ele pode ser colocado dentro de qualquer elemento, aparece após pequeno atraso perto do ponteiro e desaparece quando o ponteiro sai; para texto simples basta definir `text`, e cada elemento pode conter no máximo um Tooltip.[3]

Isso confirma que `RovexButton` pode deixar de desenhar manualmente um `Rectangle` tooltip dentro de `button-focus` e usar o elemento nativo `Tooltip { text: root.tooltip; }`. Essa solução delega o posicionamento/ordem de popup ao toolkit e evita que irmãos posteriores cubram o texto.

[3]: https://docs.slint.dev/latest/docs/slint/reference/window/tooltip/ "Slint Docs — Tooltip"

As diretrizes da Microsoft para Windows recomendam tooltips apenas quando agregam valor distinto, especialmente para controles icon-only; informação essencial deve permanecer visível na própria UI, o texto deve ser conciso e tooltips não devem conter controles interativos.[4] A Nielsen Norman Group recomenda não esconder informação vital em tooltip, evitar texto redundante, manter microconteúdo breve e suportar tanto hover do mouse quanto foco do teclado.[5]

Aplicação aos problemas encontrados: os rótulos `Rovex`, `Explorador de arquivos` e `Pronto` não são necessários para executar a tarefa principal e competem com a área útil; o cabeçalho deve ser reduzido ou removido, mantendo somente informação de estado que agrega valor. Tooltips devem permanecer nos botões icon-only de navegação/fechar/abrir/atualizar, mas aparecer fora da ordem de pintura dos irmãos e também quando o botão recebe foco.

[4]: https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/tooltips "Microsoft Learn — Tooltips - Windows apps"
[5]: https://www.nngroup.com/articles/tooltip-guidelines/ "Nielsen Norman Group — Tooltip Guidelines"

## Resultado da primeira correção

A captura do binário release confirmou que `Rovex`, `Explorador de arquivos` e `Pronto` foram removidos da área de conteúdo, deixando a janela mais limpa. Porém, o primeiro binding do Tooltip nativo foi escrito com escaping incorreto: o tooltip exibiu literalmente `{root.tooltip}`. A compilação passou, mas a inspeção visual detectou a regressão; a implementação será corrigida para usar a interpolação Slint correta antes da validação final.

## Resultado da correção final

Após corrigir o escaping para `@markdown("\{root.tooltip}")` e reconstruir o release, o tooltip de nova aba exibiu `Abrir nova aba` acima do botão, sem ser coberto pela toolbar ou pelo campo de caminho. O tooltip de atualização exibiu `Atualizar listagem` acima do botão no extremo direito, também visível e sem sobreposição incorreta. A tela principal do release novo não contém mais o cabeçalho interno redundante nem o estado solto `Pronto`.

O teste visual final confirmou que o Tooltip aninhado no FocusScope continua correto: `Abrir nova aba` aparece acima do botão `+` e `Atualizar listagem` aparece acima do botão de refresh, ambos sem serem cobertos por campos ou botões vizinhos. O posicionamento fica próximo do alvo e o texto é curto, conforme as diretrizes consultadas.
