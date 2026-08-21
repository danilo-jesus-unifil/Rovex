# Investigação e correção visual do Rovex

**Data:** 17 de agosto de 2026  
**Projeto:** Rovex v0.1.9  
**Escopo:** tooltips cobertos por controles, cabeçalho redundante, compatibilidade de plataforma e atualização dos smoke tests.

## Resultado

A causa do texto de apoio ficar atrás dos controles foi identificada no próprio código reparticionado, em `ui/components.slint`. O tooltip antigo era um `Rectangle` filho do `FocusScope` do botão. Como ele fazia parte da árvore visual normal do botão, irmãos posteriores da toolbar: como campos de caminho/filtro e o botão de atualização: podiam ser pintados acima dele. A falha era de composição e stacking order, não de Linux, X11 ou Windows.

A implementação foi substituída pelo elemento nativo `Tooltip` do Slint 1.17.1. O binding usa `@markdown("\{root.tooltip}")`, que é a conversão explícita exigida para o tipo `styled-text`, e o Tooltip fica dentro do mesmo `FocusScope` do botão. Isso preserva a associação com hover e foco de teclado, enquanto o toolkit administra o popup acima dos irmãos da interface.

A primeira tentativa compilou, mas a inspeção visual encontrou uma regressão de escaping que mostrava literalmente `{root.tooltip}`. Essa regressão foi corrigida e a captura seguinte confirmou os textos `Abrir nova aba` e `Atualizar listagem` em posição visível.

## Compatibilidade de plataforma

A pesquisa oficial do Slint confirma que o backend Winit suporta Windows, macOS e Linux com X11 ou Wayland.[1] O Rovex usa `backend-winit-x11` somente no alvo Linux porque o ambiente de testes usa Xvfb/X11; no alvo Windows usa `backend-winit` nativo. O renderer `software` é compartilhado e não exige GPU. Portanto, a seleção de feature é específica para o sistema de janelas de cada alvo, mas a arquitetura do aplicativo não é exclusiva de Linux.

As APIs condicionais do código também estão delimitadas: Registro, Known Folders e `SHGetKnownFolderPath` aparecem em módulos `cfg(windows)`; a lógica de operações, estado, jobs, conversores e handlers é compartilhada. O check `x86_64-pc-windows-gnu` continua passando após a alteração de UI.

## Simplificação da hierarquia visual

O primeiro `HorizontalLayout` da toolbar continha `Rovex`, `Explorador de arquivos` e o estado `Pronto`. Esses textos repetiam a identidade da janela e não ajudavam na tarefa de navegar, selecionar ou operar arquivos. Eles foram removidos da área de conteúdo. O título nativo da janela continua definido como `Rovex`, enquanto o status funcional continua disponível na barra inferior e nos estados de operação.

Essa decisão segue as diretrizes da Microsoft: tooltips devem ser usados com parcimônia, para informação suplementar, e não para substituir conteúdo essencial; controles icon-only são candidatos apropriados para tooltip.[2] A Nielsen Norman Group recomenda texto breve, não redundante e acessível tanto por mouse quanto por teclado.[3] A UI agora mantém tooltips apenas nos controles icon-only: voltar, avançar, subir, atualizar, abrir e fechar aba: e deixa as ações textuais visíveis diretamente.

| Elemento | Antes | Depois |
|---|---|---|
| Identidade no conteúdo | `Rovex` + `Explorador de arquivos` | Removida da toolbar; permanece o título nativo da janela |
| Estado vazio da seleção | `Pronto` solto no canto | Removido; status relevante fica na barra inferior |
| Tooltip | `Rectangle` desenhado na árvore do botão | `Tooltip` nativo do Slint, com popup acima da interface |
| Rebuild | `toolbars.slint` não era observado pelo build script | `build.rs` inclui `cargo:rerun-if-changed=ui/toolbars.slint` |
| Smoke tests | Coordenadas herdavam o cabeçalho antigo | Cliques de abas usam y=35; linha de arquivo usa y=190 |

## Validação

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 44 aprovados, 0 falhos, 2 ignorados explicitamente |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo check --target x86_64-pc-windows-gnu --all-targets --all-features` | Aprovado |
| `cargo clippy --target x86_64-pc-windows-gnu --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo build --release` e build Windows GNU | Aprovados |
| Smoke GUI, abas, menu contextual e conversão JPEG XL | Aprovados |
| Conversão com binário separado da imagem | Aprovada; `ffprobe` confirmou `jpegxl` |
| Arquivos de produção acima de 400 linhas | Nenhum |

As capturas `tooltip-fix-v0.1.9/02-tooltip-new-tab.png` e `03-tooltip-refresh.png` comprovam os tooltips corrigidos. A captura `01-main-without-header.png` comprova a hierarquia compacta sem os textos redundantes.

## Referências

[1]: https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_winit/ "Slint Docs: Winit Backend"
[2]: https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/tooltips "Microsoft Learn: Tooltips - Windows apps"
[3]: https://www.nngroup.com/articles/tooltip-guidelines/ "Nielsen Norman Group: Tooltip Guidelines"
[4]: https://docs.slint.dev/latest/docs/slint/reference/window/tooltip/ "Slint Docs: Tooltip"

## Correção adicional validada

A célula do ícone agora usa `cross-axis-alignment: center` no `HorizontalLayout` da linha, mantendo o quadrado de 34×28 px centralizado na linha de 40 px. A captura atual mostra o glyph no centro vertical do quadrado, sem ficar colado ao topo.

O Tooltip customizado agora usa `DesignTokens.text-primary` para o texto, `surface-elevated` para o painel e `border-strong` para a borda. A captura final mostra `Abrir nova aba` com a mesma cor principal de texto do software e contraste coerente com o tema escuro.
