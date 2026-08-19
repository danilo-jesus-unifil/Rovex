# Pesquisa de Terminal — 2026-08-19

## Decisão

O menu contextual oferece **Abrir Terminal aqui** como ação explícita. A ação não é executada ao listar arquivos, não altera associações do Windows e não monta uma linha de comando concatenada com entrada do usuário. O Rovex valida um diretório absoluto existente e passa o caminho como argumento separado ou como `current_dir` do processo.

| Candidato | Argumentos | Quando usar |
|---|---|---|
| Windows Terminal | `wt.exe -w new new-tab --startingDirectory <path>` | Primeira tentativa; usa a opção documentada de diretório inicial e força nova janela |
| PowerShell | `powershell.exe -NoLogo -NoExit`, `current_dir=<path>` | Fallback quando o alias do Windows Terminal não pode ser iniciado |
| Prompt de Comando | `cmd.exe /D /K`, `current_dir=<path>` | Fallback final local, disponível no Windows sem depender do Windows Terminal |

A documentação do Windows Terminal define `wt.exe`/`wt` como entrada, `new-tab` como comando padrão e `--startingDirectory`/`-d` como diretório inicial [1]. O adapter usa `.arg`/`.args` e `.current_dir` de `std::process::Command`; portanto, espaços, Unicode, `&`, `;` e outros caracteres do caminho não viram separadores de shell. Cada tentativa só cai para a seguinte quando o `spawn` falha; sucesso de `spawn` encerra a cascata e o handle do filho é liberado sem esperar o terminal fechar.

## Contrato

Se a seleção for uma pasta, o terminal inicia nela. Se for um arquivo regular, o terminal inicia no diretório pai. Uma seleção que não seja exatamente um item produz mensagem de status e nenhuma execução. Caminhos relativos, ausentes ou que não sejam diretórios são rejeitados. No Linux de desenvolvimento, o botão aparece desabilitado e o adapter retorna `Unsupported`; isso deixa a limitação explícita em vez de lançar um terminal arbitrário.

A ação é despachada em um worker nomeado (`rovex-terminal`) e o resultado volta ao event loop Slint. A UI mostra `Abrindo terminal…`, depois o candidato iniciado ou a lista limitada de falhas. O worker nunca executa `cmd /c`, `powershell -Command`, `start` ou uma string construída com o caminho; também não envia arquivos a serviço externo.

## Limitações

O CI Windows atual é não interativo: ele compila, testa e executa o smoke CLI nativo, mas não possui uma sessão visual que possa confirmar a janela do Windows Terminal. Os testes locais sob Xvfb verificam que o botão existe, permanece no tema escuro e fica desabilitado no Linux; testes unitários verificam seleção de diretório e separação dos argumentos Windows. A validação visual efetiva dos três candidatos no Windows 10/11, incluindo aliases do Windows Terminal e políticas corporativas, requer uma sessão Windows interativa.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/terminal/command-line-arguments "Windows Terminal command line arguments — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/shell/launch "Launching Applications (ShellExecute, ShellExecuteEx, SHELLEXECUTEINFO) — Microsoft Learn"

[3]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw "ShellExecuteExW function — Microsoft Learn"
