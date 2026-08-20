# Rovex v0.1.15 — Release report

## Resumo

A versão `v0.1.15` adiciona **Abrir Terminal aqui** ao menu contextual. A ação é explícita: não é executada ao listar arquivos, não altera associações de arquivos e não executa comandos derivados do nome do item. Uma pasta selecionada é usada como diretório inicial; um arquivo selecionado usa apenas seu diretório pai. O botão permanece desabilitado no Linux de desenvolvimento, pois o adapter desta release é direcionado ao Windows.

| Aspecto | Contrato implementado |
|---|---|
| Windows Terminal | `wt.exe -w new new-tab --startingDirectory <path>` |
| Fallback 1 | `powershell.exe -NoLogo -NoExit`, com `current_dir` |
| Fallback 2 | `cmd.exe /D /K`, com `current_dir` |
| Segurança de argumentos | `Command::args` e `Command::current_dir`; nenhum `cmd /c`, `powershell -Command`, `start` ou string concatenada |
| Execução | Worker `rovex-terminal`; o handle é liberado sem esperar o terminal fechar |
| Validação | Caminho absoluto, existente, diretório real e sem symlink/reparse point |
| Feedback | `Abrindo terminal…`, candidato iniciado ou erro limitado retornam pelo event loop Slint |

## Implementação e segurança

O adapter em `src/terminal.rs` separa a seleção do diretório, a composição dos candidatos Windows e o spawn. O handler em `src/desktop/handlers/terminal.rs` exige exatamente uma seleção, fecha o menu, inicia o worker e publica o resultado na UI. Nenhum caminho é interpretado por um shell intermediário. Unicode, espaços, ponto e vírgula, ampersand e demais caracteres permanecem dados de um argumento.

A revisão posterior ao primeiro CI encontrou um risco de redirecionamento especial: um diretório symlink/reparse point poderia fazer a ação alcançar um destino diferente do apresentado. A correção usa `symlink_metadata` e rejeita o diretório final quando ele é symlink no host Unix ou possui `FILE_ATTRIBUTE_REPARSE_POINT` no Windows. O teste Unix `symlink_target_is_rejected_without_following_it` impede regressão; o código Windows passa pelo cross-check e pela suíte nativa, mas criação de symlink sem privilégio não é forçada em runner.

> A cascata só tenta o próximo candidato quando o `spawn` falha. Depois que um processo foi iniciado, não há segunda tentativa automática.

O menu contextual recebeu rolagem interna e altura limitada pela janela, mantendo as superfícies, bordas, raios, estados disabled e contraste do tema escuro. A correção no `scripts/smoke_gui.sh` também garante o PATH padrão do Cargo antes de chamar `xvfb-run`, eliminando o falso erro de status 127 observado quando o ambiente não exportava `~/.cargo/bin`.

## Validação

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Passou |
| `cargo test --all-targets --all-features` | 94 passaram; 2 ignorados explícitos |
| Clippy host com `-D warnings` | Passou |
| Check e Clippy Windows GNU | Passaram |
| Link dos testes Windows GNU | Passou |
| `cargo audit` e `cargo deny` | Passaram no CI |
| Build release host | Passou |
| Cross-build Windows release | Passou |
| Smoke gráfico principal | Passou; depois da correção, também sem PATH manual |
| Captura do menu contextual | Confirmou botão Terminal desabilitado no Linux, tema escuro, layout rolável e ausência de clipping |
| CI `windows-latest` | Passou em `ab64b9f`: testes, Clippy, release e smoke CLI nativo |

## Limitações honestas

O runner Windows comprova compilação, testes, Clippy, build release e smoke CLI nativo, mas não abre uma sessão visual para confirmar a janela de cada candidato. A validação interativa de `wt.exe` e aliases de execução, PowerShell, Prompt de Comando, políticas corporativas, Windows 10 22H2 separado de `windows-latest`, DPI, acessibilidade, alto contraste, UNC/SMB, volumes removíveis e caminhos extremos permanece pendente. A release continua sem MSI/MSIX, assinatura Authenticode e atualizador automático.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/terminal/command-line-arguments "Windows Terminal command line arguments — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/shell/launch "Launching Applications (ShellExecute, ShellExecuteEx, SHELLEXECUTEINFO) — Microsoft Learn"

[3]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw "ShellExecuteExW function — Microsoft Learn"
