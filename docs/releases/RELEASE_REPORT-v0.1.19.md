# Rovex v0.1.19 — contrato seguro de ativação Windows

Data: 2026-08-20

A v0.1.19 corrige uma lacuna confirmada no adapter de ativação introduzido na v0.1.18. O Rovex chamava `ShellExecuteExW` em um worker nomeado sem message loop, mas deixava `SHELLEXECUTEINFO.fMask` em zero. A documentação oficial exige `SEE_MASK_NOASYNC` quando a thread chamadora não possui message loop ou não permanecerá disponível para concluir a conversa DDE. O mesmo contrato recomenda inicializar COM com `COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE`.

| Item | Resultado |
|---|---|
| Versão | `0.1.19` |
| Branch de backup | `backup/before-activation-thread-contract-20260820` |
| Adapter | `src/activation.rs` |
| Correção | `SEE_MASK_NOASYNC` e `COINIT_DISABLE_OLE1DDE` |
| Erro tipado | `SE_ERR_*` em `hInstApp` + `GetLastError` |
| Gate | `scripts/test_activation_contract.sh` ampliado |
| Pesquisa | `docs/research/activation-errors-research-2026-08-20.md` |

## Auditoria e falha confirmada

O roadmap do issue #2 listava associação inexistente, arquivo bloqueado, junctions, caminhos UNC e paths longos como riscos a investigar, sem declarar resolução por cross-build. A inspeção do código confirmou uma falha mais fundamental e reproduzível sem depender de uma máquina Windows específica: o worker que chama `ShellExecuteExW` não possui message loop e não configurava a flag de sincronização recomendada para esse caso.

A documentação oficial de `SHELLEXECUTEINFO` afirma que `SEE_MASK_NOASYNC` deve ser usado se a thread chamadora não possui message loop ou não permanecerá disponível para finalizar a conversa DDE. A referência também recomenda `COINIT_DISABLE_OLE1DDE` junto de `COINIT_APARTMENTTHREADED`. A comparação com o código v0.1.18 confirmou a divergência; a correção foi implementada diretamente no adapter e coberta por gate estrutural.

> O cross-build comprova ABI e compilação, mas não substitui a execução nativa de associações, ACLs, arquivos bloqueados, junctions ou UNC. Esses casos continuam explicitamente pendentes até reprodução controlada em Windows.

## Correção implementada

`SHELLEXECUTEINFOW.fMask` agora recebe `SEE_MASK_NOASYNC`. A inicialização COM usa `COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE`, preserva `CoUninitialize` apenas quando a inicialização foi bem-sucedida e continua executando fora do thread visual.

Quando `ShellExecuteExW` falha, o adapter captura `GetLastError` imediatamente e preserva também o valor de `hInstApp`, que carrega o código compatível `SE_ERR_*`. `ActivationError::ShellExecuteFailed` agora mantém os dois códigos. Mensagens controladas distinguem arquivo/caminho ausente, acesso negado, compartilhamento, associação inexistente e cancelamento. Nenhum erro tenta `Command`, `cmd.exe`, `runas`, `Open With` ou qualquer fallback de shell.

A política de reparse points não foi relaxada. A pesquisa oficial confirma que reparse points incluem links NTFS, mounted folders e filtros de filesystem, não apenas symlinks; o bloqueio por atributo em cada componente normal continua sendo a opção conservadora para uma ação de ativação.

## Evidências de validação

| Verificação | Resultado |
|---|---|
| Testes host | 103 aprovados; 2 ignorados explicitamente |
| Mensagem de associação inexistente | Testada com `ERROR_NO_ASSOCIATION`/1155 |
| Mensagem de acesso negado | Testada |
| Mensagem de compartilhamento | Testada |
| Mensagem de cancelamento | Testada |
| Gate de `SEE_MASK_NOASYNC` | Aprovado |
| Gate de `COINIT_DISABLE_OLE1DDE` | Aprovado |
| Gate de ponteiros Shell nulos | Aprovado |
| Gate contra `Command::new` | Aprovado |
| Check/Clippy host | Aprovados sem warnings |
| Check/Clippy/build release Windows GNU | Aprovados |
| Gates documental e de distribuição | Reexecutados antes da publicação |

## Limitações honestas

A v0.1.19 não declara que toda associação Windows funciona. Associação inexistente, acesso negado real, compartilhamento, junctions, mounted folders, caminhos UNC e paths longos ainda precisam de fixtures nativas em Windows 10/11. O erro agora chega ao usuário com códigos suficientes para diagnóstico, mas o Rovex não modifica o Registro, não cria associações, não eleva privilégios e não tenta executar conteúdo por caminhos alternativos.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexa "ShellExecuteExA function"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-shellexecuteinfoa "SHELLEXECUTEINFOA structure"
[3]: https://learn.microsoft.com/en-us/windows/win32/shell/launch "Launching Applications"
[4]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points "Reparse Points"
[5]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-point-tags "Reparse Point Tags"
[6]: https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--1000-1299- "System Error Codes 1000-1299"
