# Rovex v0.1.18 — ativação explícita de arquivos

Data: 2026-08-20

A v0.1.18 corrige uma lacuna real de interação no núcleo do Rovex: o callback de ativação de uma linha existia e era conectado ao markup Slint, mas o handler só navegava para diretórios e retornava sem ação para arquivos regulares. O menu contextual também oferecia `Abrir com...`, mas não uma ação explícita para o aplicativo padrão.

| Item | Resultado |
|---|---|
| Versão | `0.1.18` |
| Branch de backup | `backup/before-explicit-activation-20260820` |
| Adapter | `src/activation.rs` |
| Handler UI | `src/desktop/handlers/activation.rs` |
| API Windows | `ShellExecuteExW` com `lpVerb = NULL` |
| Gate adicional | `scripts/test_activation_contract.sh` |
| Pesquisa | `docs/research/activation-research-2026-08-20.md` |

## Auditoria e falha confirmada

A auditoria do issue #2 e do roadmap identificou o próximo lote como a consolidação da ativação explícita. A inspeção confirmou que `RovexFileList` emitia `activate(int)` em duplo clique e na tecla Enter, mas `navigation.rs` descartava imediatamente qualquer `FileRow` que não fosse diretório. A inspeção do menu contextual confirmou que não havia botão `Abrir` separado de `Abrir com...`.

A falha foi reproduzível no contrato do código: selecionar um arquivo regular acionava o callback, porém nenhuma API de abertura era chamada e nenhum status de erro era produzido. A hipótese de risco de execução arbitrária também foi pesquisada nas APIs oficiais do Windows antes da implementação; a decisão foi usar o Shell com verbo padrão, sem montar comando externo.

> A ativação não é confundida com navegação, Open With ou Terminal. Cada ação possui um contrato separado e uma validação própria.

## Pesquisa técnica

A documentação oficial da Microsoft descreve `ShellExecuteEx` como a API para agir sobre um arquivo e informa que `lpVerb = NULL` solicita o verbo padrão do tipo de arquivo. A mesma documentação diferencia `open`, `edit`, `print` e `runas`, e recomenda COM inicializado porque extensões do Shell podem usar COM. A documentação de verbos confirma que o caminho deve ser tratado como o item do Shell, não como uma linha de comando construída pela aplicação.

A documentação de reparse points confirma que o sistema de arquivos pode redirecionar a abertura por filtros e links NTFS. Por isso, o adapter rejeita o arquivo final e também cada componente normal do caminho que seja symlink ou reparse point. Caminhos com `..` também são rejeitados para impedir ambiguidade do alvo.

## Correção implementada

`src/activation.rs` valida que o caminho é absoluto, existe como arquivo regular, não contém `..` e não atravessa symlinks/reparse points. No Windows, `activate_file` inicializa COM em STA, cria `SHELLEXECUTEINFOW` com o caminho UTF-16 em `lpFile`, deixa `lpVerb`, `lpParameters` e `lpDirectory` nulos e chama `ShellExecuteExW` sem executar um shell externo. O resultado de `GetLastError` é convertido para `ActivationError` quando a chamada falha.

`src/desktop/handlers/activation.rs` mantém a UI responsiva: a ativação ocorre em worker nomeado e o status retorna ao event loop Slint. O helper é usado tanto pelo botão contextual quanto pelo duplo clique/Enter. Pastas preservam navegação interna. Links simbólicos, itens especiais e diretórios não são ativados.

O menu contextual recebeu `Abrir` e a flag `can-open`. `Abrir com...` continua sendo o diálogo `SHOpenWithDialog` independente. Os resets dos diálogos de operação e conversão foram atualizados para impedir flags residuais.

## Testes e gates

| Verificação | Resultado |
|---|---|
| Arquivo absoluto regular | Aprovado |
| Diretório, caminho relativo e ausente | Rejeitados |
| Symlink no arquivo final | Rejeitado |
| Arquivo dentro de pai symlinkado | Rejeitado |
| Caminho com `..` | Rejeitado |
| `ShellExecuteExW` e contrato de `lpFile` | Verificados pelo gate estrutural |
| Construção de `Command::new` no adapter | Ausente e bloqueada pelo gate |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 102 aprovados; 2 ignorados |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| Check/Clippy/build release Windows GNU | Aprovados |
| Smoke gráfico, contexto, Open With e abas | Aprovados |
| Gate Markdown e links locais | Aprovado |

Durante o ciclo, o primeiro Clippy cross-Windows encontrou `field_reassign_with_default` na construção de `SHELLEXECUTEINFOW`. A causa foi corrigida com inicialização direta usando `..Default::default()`, e o check cross, Clippy cross e build release foram repetidos com sucesso. Também houve uma falha operacional de espaço em disco causada por 36 GB de caches regeneráveis em `target`; o diretório foi removido sem tocar no código, e toda a validação foi repetida com 31 GB livres.

## Limitações honestas

O Linux não possui o Shell Windows, portanto a API fica desabilitada nesse alvo e o smoke gráfico Linux valida somente o estado visual e o contrato de seleção. O CI Windows valida compilação, testes `cfg(windows)` e o contrato estrutural, mas a associação efetiva de cada extensão depende das associações configuradas na sessão Windows do usuário. A feature não cria associações, não eleva privilégios, não usa `runas`, não baixa executáveis e não transforma arquivos em comandos.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/shell/launch "Launching Applications"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutea "ShellExecuteA function"
[3]: https://learn.microsoft.com/en-us/windows/win32/shell/fa-verbs "Verbs and File Associations"
[4]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points "Reparse Points"
[5]: https://github.com/danilo-jesus-unifil/Rovex/issues/2 "Rovex issue #2"
