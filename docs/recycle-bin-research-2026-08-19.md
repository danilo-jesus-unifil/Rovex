# Pesquisa de integração com a Lixeira — 2026-08-19

## APIs consideradas

A documentação oficial informa que `SHFileOperation` envia um item para a Lixeira quando `FOF_ALLOWUNDO` está presente; sem esse flag, uma exclusão é permanente [1]. A mesma página recomenda nomes de caminho totalmente qualificados, informa que a API foi substituída por `IFileOperation` no Windows Vista e registra `DE_PATHTOODEEP`/`DE_ERROR_MAX` entre os códigos possíveis, portanto a rota Shell não é uma garantia de suporte a todo caminho longo.

`IFileOperation` é a interface mais nova e expõe `IShellItem`, resultados HRESULT mais precisos, progresso e operações combinadas [2]. Seu método `SetOperationFlags` define `FOFX_RECYCLEONDELETE`, introduzido no Windows 8, para enviar exclusões à Lixeira [3]. Ela exige uma camada COM mais extensa e não foi ativada neste lote porque a fundação atual usa workers Rust simples e a operação já possui confirmação, cancelamento entre itens e refresh próprios.

| Opção | Decisão | Justificativa |
|---|---|---|
| `SHFileOperationW` | Implementada no alvo Windows | API disponível no Windows 10, caminho UTF-16, `FOF_ALLOWUNDO`, integração pequena e verificável. |
| `IFileOperation` | Reservada para evolução | Melhor API e progresso Shell, mas exige COM/`IShellItem`/HRESULT e uma estratégia de cancelamento adicional. |
| `DeleteFileW`/`RemoveDirectoryW` | Não usada no Windows | Excluiria permanentemente e violaria a expectativa de Lixeira. |
| Shell externo via comando | Rejeitada | Não há shell para montar comando, nem download/execução de executáveis em runtime. |

## Contrato adotado

O adapter chama a operação uma entrada por vez, com buffer UTF-16 terminado por dois NULs, `FOF_ALLOWUNDO`, `FOF_NOCONFIRMATION`, `FOF_NOERRORUI`, `FOF_SILENT` e `FOF_NORECURSION`. A confirmação continua sendo responsabilidade da UI; o Shell não abre uma segunda confirmação. O resultado não-zero e `fAnyOperationsAborted` viram erro estruturado, e o Rovex não faz fallback silencioso para exclusão permanente. Se a Lixeira estiver indisponível, o item permanece no filesystem e a UI relata a falha.

A camada existente ainda bloqueia raízes, valida a origem via `symlink_metadata` e preserva o contrato de não excluir diretórios não vazios. No Windows, o diretório é inspecionado antes e a chamada Shell usa `FOF_NORECURSION`; no Unix, o comportamento anterior de remoção permanente continua sendo o fallback de desenvolvimento. Cada item é processado individualmente, de modo que cancelamento entre itens continua possível, embora uma chamada Shell individual não possa ser interrompida pelo `AtomicBool` no meio da operação.

## Limitações verificáveis

O cross-check Windows GNU compila a chamada e seus bindings, mas esta sessão não possui Windows 10/11 nativo nem uma Lixeira real para validar restauração, volumes removíveis, UNC/SMB, políticas de grupo, arquivos em uso, ACLs, Shell extensions ou paths maiores que os limites aceitos pela API. O projeto deve preferir reportar falha e preservar o arquivo nessas situações; não deve declarar que a operação foi concluída quando o Shell a abortou.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shfileoperationa "SHFileOperationA function — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileoperation "IFileOperation interface — Microsoft Learn"

[3]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags "IFileOperation::SetOperationFlags method — Microsoft Learn"
