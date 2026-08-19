# Rovex v0.1.14 — Release report

## Resumo

A versão `v0.1.14` evolui a exclusão para a Lixeira no Windows de `SHFileOperationW` para `IFileOperation`/COM, preservando a API anterior como fallback somente durante a preparação da operação. O objetivo é usar a representação mais robusta `IShellItem`, flags modernas de reciclagem e HRESULTs estruturados sem mover COM para a UI, sem chamar shell externo e sem transformar uma falha parcial em nova tentativa potencialmente destrutiva.

| Item | Implementação |
|---|---|
| Representação do item | `SHCreateItemFromParsingName` cria `IShellItem` a partir de caminho UTF-16 terminado por um NUL |
| Operação | `CoCreateInstance` cria `IFileOperation` em apartment COM por chamada de worker |
| Lixeira | `FOFX_RECYCLEONDELETE` com `FOFX_EARLYFAILURE`, `FOF_NOERRORUI`, `FOF_NOCONFIRMATION`, `FOF_NORECURSION` e `FOF_SILENT` |
| Fallback | `SHFileOperationW` com `FOF_ALLOWUNDO` somente se COM, parsing, flags ou enfileiramento falharem antes de `PerformOperations` |
| Falha após início | HRESULT de `PerformOperations` ou aborto do Shell retorna erro; não há repetição automática |
| Plataforma | Código e bindings ficam em `src/operations/recycle.rs` sob `cfg(windows)` |

## Segurança e ciclo de vida

A validação existente continua anterior ao adapter: raízes são recusadas, a origem é inspecionada sem seguir symlink/reparse point e diretórios não vazios continuam rejeitados. O worker declara o item e somente depois chama `PerformOperations`. O `ComApartment` desfaz `CoInitializeEx` apenas quando a própria chamada inicializou COM; `RPC_E_CHANGED_MODE` não provoca `CoUninitialize` indevido. Os ponteiros retornados por COM possuem guardas RAII que chamam `Release` exatamente uma vez.

> O fallback é delimitado por estado: preparação pode ser repetida; execução iniciada não pode ser repetida automaticamente.

O adapter não contém concatenação de comandos, não executa arquivo listado, não baixa executáveis e não solicita elevação. A confirmação segue na UI; as flags impedem uma confirmação duplicada do Shell. No Linux, o comportamento de remoção permanente de desenvolvimento permanece inalterado e não é apresentado como semântica da Lixeira Windows.

## Testes e evidências

A implementação foi submetida ao ciclo incremental após falhas reais de compilação e lint: primeiro o check Windows revelou imports de GUIDs não expostos, casts de interface, match não exaustivo e `GUID` sem `PartialEq`; cada causa foi corrigida e coberta por testes. Os testes verificam terminadores UTF-16, GUIDs oficiais por campos, flags de reciclagem, distinção entre falha de preparação e falha pós-execução e preservação de HRESULT em `OperationError::FileSystem`.

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Passou |
| `cargo test --all-targets --all-features` | 90 passaram; 2 ignorados explícitos |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passou |
| Clippy Windows GNU estrito | Passou |
| `cargo check` e link de testes Windows GNU | Passaram |
| `cargo audit` e `cargo deny check advisories licenses bans sources` | Passaram |
| Builds release host e Windows GNU | Passaram |
| Manifesto PE | Validado |
| Smoke GUI, navegação, operações, preview, settings e conversões | Passaram |
| CI `windows-latest` | Passou: testes, Clippy, release e smoke CLI nativo |

## Limitações

O CI Windows comprova execução nativa não interativa da suíte, incluindo o fluxo de operações de arquivo, mas não fixa neste repositório a versão exata do Windows nem substitui uma sessão interativa. Ainda não há evidência específica de restauração visual na Lixeira, Windows 10 22H2 separado de `windows-latest`, drag-and-drop efetivo do Explorer, ACLs reais, DPI, leitor de tela, alto contraste, UNC/SMB, volumes removíveis, reparse points complexos, arquivos em uso ou paths extremos. A release continua sem instalador MSI/MSIX, assinatura Authenticode e atualizador automático; não há certificado ou integração simulados.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileoperation "IFileOperation interface — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags "IFileOperation::SetOperationFlags method — Microsoft Learn"

[3]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromparsingname "SHCreateItemFromParsingName function — Microsoft Learn"

[4]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ishellitem "IShellItem interface — Microsoft Learn"
