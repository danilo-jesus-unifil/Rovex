# Ativação Windows: sincronização de thread e diagnóstico de erros

Data: 2026-08-20

## Falha confirmada

A v0.1.18 chamava `ShellExecuteExW` em um worker nomeado que não possui message loop, mas deixava `SHELLEXECUTEINFOW.fMask` em zero. A documentação oficial determina que `SEE_MASK_NOASYNC` deve ser usado quando a thread chamadora não possui message loop ou não permanecerá disponível para concluir uma conversa DDE. A lacuna foi confirmada comparando o contrato documentado com o código efetivo; não depende de uma associação específica do Windows.

O mesmo contrato recomenda inicializar COM com `COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE` para extensões do Shell que usam COM. O código anterior usava somente `COINIT_APARTMENTTHREADED`. O adapter agora declara ambos os contratos e continua fora do thread visual.

## Diagnóstico de falhas

`ShellExecuteExW` retorna `FALSE` em falha, fornece um código compatível `SE_ERR_*` em `hInstApp` e permite obter informação mais precisa via `GetLastError`. O adapter agora preserva ambos os códigos em `ActivationError::ShellExecuteFailed` e converte casos conhecidos em mensagens controladas: arquivo/caminho ausente, acesso negado, compartilhamento, associação inexistente e cancelamento.

Nenhum desses erros dispara fallback para `Command`, `cmd.exe`, `runas` ou execução automática. A ação continua sendo uma ativação pelo Shell com verbo padrão e sem parâmetros construídos pela aplicação.

## Reparse points

Reparse point não é sinônimo de symlink. A documentação oficial cita links NTFS, mounted folders e filtros de filesystem; o tag define o comportamento interpretado pelo sistema. Por isso, a validação conservadora do Rovex continua recusando `FILE_ATTRIBUTE_REPARSE_POINT` no arquivo e nos componentes normais do caminho. Relaxar essa regra exigiria testes nativos de cada tag suportada e um contrato específico de segurança.

## Gates incrementados

| Gate | Cobertura |
|---|---|
| Testes unitários | Mensagens para associação inexistente, acesso negado, compartilhamento e cancelamento |
| Gate estrutural | `SEE_MASK_NOASYNC`, `COINIT_DISABLE_OLE1DDE`, `fMask`, ponteiros nulos e erro tipado |
| Cross-Windows | Check, Clippy com warnings como erros e build release GNU |
| CI | O mesmo gate roda em Ubuntu e Windows nativo |

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexa "ShellExecuteExA function"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-shellexecuteinfoa "SHELLEXECUTEINFOA structure"
[3]: https://learn.microsoft.com/en-us/windows/win32/shell/launch "Launching Applications"
[4]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points "Reparse Points"
[5]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-point-tags "Reparse Point Tags"
[6]: https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--1000-1299- "System Error Codes 1000-1299"
