# Ativação explícita de arquivos no Windows — pesquisa e contrato

Data: 2026-08-20

## Fontes oficiais

- [Microsoft — Launching Applications](https://learn.microsoft.com/en-us/windows/win32/shell/launch)
- [Microsoft — ShellExecuteA](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutea)
- [Microsoft — Verbs and File Associations](https://learn.microsoft.com/en-us/windows/win32/shell/fa-verbs)
- [Microsoft — Reparse Points](https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points)

## Falha confirmada

Antes deste lote, o evento `activate(int)` navegava para diretórios, mas retornava sem ação para arquivos regulares. O menu contextual também não tinha uma ação explícita de abertura pelo aplicativo padrão. Isso tornava o comportamento de duplo clique e Enter incompleto: a mesma ação visual podia selecionar um arquivo sem abrir seu handler associado.

A lacuna foi confirmada no código, no markup Slint e no contrato de seleção. Não foi tratada como hipótese de UX: o evento existia, era ligado ao Rust, e o handler deliberadamente descartava qualquer linha que não fosse diretório.

## Contrato implementado

O Rovex agora usa um adapter dedicado para arquivos regulares. No Windows, a chamada usa `ShellExecuteExW` com `lpVerb = NULL`, deixando o Shell escolher o verbo padrão do tipo de arquivo. O caminho absoluto é entregue em `lpFile`; `lpParameters` e `lpDirectory` permanecem nulos, portanto nenhuma linha de comando é montada e nenhum shell externo é invocado.

A chamada ocorre em worker nomeado e com COM STA inicializado. O resultado volta para a UI por `slint::invoke_from_event_loop`. Diretórios continuam sendo navegados dentro do Rovex, Open With continua sendo um diálogo separado e Terminal continua sendo uma ação diferente.

A validação recusa caminhos relativos, caminhos ausentes, diretórios, symlinks e reparse points. Arquivos regulares normais e ocultos podem ser ativados; links simbólicos e itens especiais permanecem fora do contrato.

## Validações incrementadas

| Gate | Cobertura |
|---|---|
| Testes unitários do adapter | Arquivo absoluto regular, diretório, caminho relativo, caminho ausente e symlink |
| Gate `test_activation_contract.sh` | Testes do adapter, callback Slint distinto, `ShellExecuteExW`, `lpFile` e ausência de `Command::new` |
| CI | O gate roda em Ubuntu e Windows além da suíte completa, Clippy e builds release |
| Fluxo UI | Duplo clique e Enter ativam arquivos regulares; pastas navegam; itens especiais exibem status sem executar |

## Limites conhecidos

O teste automatizado não abre um aplicativo associado dentro do Xvfb Linux, porque a API Shell usada pelo contrato é específica do Windows. O CI Windows valida compilação, testes `cfg(windows)` e o contrato estrutural; a execução interativa de associação padrão depende da sessão do usuário e da associação configurada no host Windows.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/shell/launch "Launching Applications"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutea "ShellExecuteA function"
[3]: https://learn.microsoft.com/en-us/windows/win32/shell/fa-verbs "Verbs and File Associations"
[4]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points "Reparse Points"
