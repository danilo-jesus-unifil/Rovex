# Pesquisa de integração com a Lixeira — 2026-08-19

## APIs consideradas

A documentação oficial informa que `SHFileOperation` envia um item para a Lixeira quando `FOF_ALLOWUNDO` está presente; sem esse flag, uma exclusão é permanente [1]. A mesma página recomenda nomes de caminho totalmente qualificados, informa que a API foi substituída por `IFileOperation` no Windows Vista e registra `DE_PATHTOODEEP`/`DE_ERROR_MAX` entre os códigos possíveis, portanto a rota Shell não é uma garantia de suporte a todo caminho longo.

`IFileOperation` é a interface mais nova e expõe `IShellItem`, resultados HRESULT mais precisos, progresso e operações combinadas [2]. Seu método `SetOperationFlags` define `FOFX_RECYCLEONDELETE`, introduzido no Windows 8, para enviar exclusões à Lixeira [3]. Ela agora é a rota principal do adapter Windows: uma chamada COM por item, sem diálogo do Shell, dentro do worker de operações existente. O progresso visual e o cancelamento entre itens continuam sob responsabilidade do Rovex, enquanto o COM não é mantido na UI.

| Opção | Decisão | Justificativa |
|---|---|---|
| `IFileOperation` | Rota principal no alvo Windows | API recomendada pela Microsoft, `IShellItem` por caminho UTF-16, HRESULT e `FOFX_RECYCLEONDELETE`; bindings COM mínimos ficam isolados no adapter. |
| `SHFileOperationW` | Fallback somente de preparação | Mantida para indisponibilidade de COM, falha de criação/parsing/configuração antes de `PerformOperations`; não é usada após uma mutação parcial. |
| `DeleteFileW`/`RemoveDirectoryW` | Não usada no Windows | Excluiria permanentemente e violaria a expectativa de Lixeira. |
| Shell externo via comando | Rejeitada | Não há shell para montar comando, nem download/execução de executáveis em runtime. |

## Contrato adotado

O adapter chama `SHCreateItemFromParsingName` com um caminho UTF-16 terminado por um NUL, cria `IFileOperation` via COM e configura `FOFX_RECYCLEONDELETE`, `FOFX_EARLYFAILURE`, `FOF_NOCONFIRMATION`, `FOF_NOERRORUI`, `FOF_NOCONFIRMMKDIR`, `FOF_SILENT` e `FOF_NORECURSION`. A confirmação continua sendo responsabilidade da UI; o Shell não abre uma segunda confirmação. HRESULTs negativos de `PerformOperations` e `GetAnyOperationsAborted` viram erro estruturado. Falhas anteriores à execução usam `SHFileOperationW` com buffer de dois NULs; nenhum caminho cai em exclusão permanente silenciosa. Se a Lixeira estiver indisponível ou a operação for abortada, o item permanece preservado conforme o resultado informado pelo Shell.

A camada existente ainda bloqueia raízes, valida a origem via `symlink_metadata` e preserva o contrato de não excluir diretórios não vazios. No Windows, o diretório é inspecionado antes e a chamada Shell usa `FOF_NORECURSION`; no Unix, o comportamento anterior de remoção permanente continua sendo o fallback de desenvolvimento. Cada item é processado individualmente, de modo que cancelamento entre itens continua possível, embora uma chamada Shell individual não possa ser interrompida pelo `AtomicBool` no meio da operação.

## Limitações verificáveis

O cross-check Windows GNU compila a chamada e os bindings. O runner `windows-latest` também executa a suíte nativa, incluindo operações de arquivo, mas não verifica visualmente a restauração na Lixeira nem fixa a versão exata do Windows. Restam validações interativas de restauração, volumes removíveis, UNC/SMB, políticas de grupo, arquivos em uso, ACLs, Shell extensions e paths maiores que os limites aceitos pela API. O projeto deve preferir reportar falha e preservar o arquivo nessas situações; não deve declarar que a operação foi concluída quando o Shell a abortou.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shfileoperationa "SHFileOperationA function — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileoperation "IFileOperation interface — Microsoft Learn"

[3]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags "IFileOperation::SetOperationFlags method — Microsoft Learn"
