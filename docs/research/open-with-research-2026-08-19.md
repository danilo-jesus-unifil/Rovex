# Pesquisa Open With — 2026-08-19

## Decisão

O Rovex implementa **Abrir com...** como abertura do diálogo nativo `SHOpenWithDialog`, e não como ShellExecute com verbo padrão. Isso mantém a escolha do aplicativo explícita e evita que a navegação execute automaticamente a associação atual do arquivo.

| Elemento | Contrato |
|---|---|
| API | `SHOpenWithDialog(HWND, const OPENASINFO*)`, exportada por Shell32 |
| Arquivo | Um único arquivo regular, absoluto e existente |
| Tipo | `pcszClass = NULL`, permitindo que o Shell use a extensão do arquivo |
| Execução | `OAIF_EXEC` (`0x4`) solicita abrir o arquivo após a escolha do usuário |
| Registro | Nenhuma flag de alteração de associação é enviada; no Windows 10 o diálogo não altera o programa padrão |
| COM | `CoInitializeEx(NULL, COINIT_APARTMENTTHREADED)` em worker próprio; `CoUninitialize` somente quando a chamada inicializa o apartment |
| UI | Botão contextual habilitado apenas para arquivo regular no Windows; Linux mantém o botão desabilitado |

A Microsoft documenta que `SHOpenWithDialog` exibe o diálogo Open With e retorna `HRESULT`; também informa que, no Windows 10, flags de registro são ignoradas e que a API deve abrir um único arquivo [1]. A estrutura `OPENASINFO` recebe caminho UTF-16, classe opcional e flags [2]. O Rovex passa um buffer UTF-16 terminado por NUL e o caminho nunca é concatenado em comando.

## Segurança e concorrência

O handler exige exatamente uma seleção e rejeita pasta, link, reparse point ou item inexistente antes de iniciar o worker. O diálogo é chamado em thread nomeada (`rovex-open-with`), não na thread visual. O `HWND` pai é nulo para deixar o Shell posicionar o diálogo; não há elevação, `runas`, `rundll32`, `cmd /c`, `powershell -Command` ou execução silenciosa.

O flag `OAIF_EXEC` não escolhe um aplicativo sozinho: ele somente permite que a seleção feita pelo usuário no diálogo seja aplicada ao arquivo. A ação não é disparada por listar diretórios nem pelo duplo clique; o callback de ativação existente continua navegando apenas para pastas.

## Testes

Testes unitários cobrem arquivo absoluto regular com Unicode/espaço, pasta, caminho relativo, arquivo ausente e symlink. O cross-check Windows GNU e o link dos binários de teste validam a declaração `OPENASINFO`, a ligação Shell32 e o caminho COM. O smoke gráfico de arquivo regular confirma a posição do botão, preview e tema; no Linux o disabled é esperado. O CI Windows precisa validar a abertura visual do diálogo em uma sessão interativa, pois o runner atual executa somente testes e smoke CLI nativo.

## Limitações

O diálogo pode depender de políticas, apps instalados e UX específica do Windows 10/11. Esta implementação não enumera aplicativos, não grava associações, não implementa editor próprio e não promete que haverá um candidato disponível para qualquer extensão. A verificação interativa do fluxo selecionar-aplicativo–abrir-arquivo permanece pendente em Windows real.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shopenwithdialog "SHOpenWithDialog function — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ns-shlobj_core-openasinfo "OPENASINFO structure — Microsoft Learn"
