# Rovex v0.1.13 — Release report

## Resumo

A versão `v0.1.13` é uma correção orientada por evidência do primeiro runner Windows nativo do projeto. O CI encontrou e o código corrigiu três problemas que não apareciam no host Linux: nomes reservados Windows eram aceitos pela API de criação, o round-trip de settings usava um caminho `/tmp` que não é absoluto no Windows e gravações concorrentes colidiam durante `MoveFileExW`.

| Item | Correção |
|---|---|
| Nomes reservados | Validação explícita de `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `COM1`–`COM9` e `LPT1`–`LPT9`, com extensão/pontuação/espaços finais |
| Caminhos | Teste de round-trip usa caminho absoluto nativo `C:\Rovex\ação segura` no Windows |
| Concorrência | Mutex de processo serializa temporário + replace do SettingsStore |
| CI nativo | Job `windows-latest` executa qualidade Rust, release e smoke CLI PowerShell |
| Assinatura | Continua ausente; nenhum certificado é simulado |

## Evidência do runner

O primeiro job Windows nativo falhou conscientemente em `nomes_reservados_do_windows_sao_rejeitados_pelo_sistema`, `round_trips_preferences_and_unicode_path` e `concurrent_saves_leave_a_valid_complete_file`. As três falhas foram corrigidas e testadas no host/cross-build; o workflow subsequente deve ser o gate nativo final desta release. O smoke CLI cria `arquivo ação seguro.txt` e `pasta com espaço` no diretório temporário do runner, executa `cargo run --quiet -- --cli`, verifica código zero e exige que ambos os nomes apareçam na saída.

A correção de nomes acontece antes de qualquer syscall de criação ou rename. A regra trata o componente final de modo case-insensitive, remove apenas pontuação/espaços finais equivalentes ao namespace Windows e analisa a parte antes do primeiro ponto. Em settings, o lock é mantido somente durante a criação/sincronização/substituição do arquivo; ele não atravessa leitura de UI nem bloqueia workers externos.

## Limitações

O job nativo comprova compilação, testes Rust, Clippy, execução do CLI e preservação básica de nomes. Ele não substitui execução interativa em Windows 10 22H2/Windows 11, validação de DPI, Explorer drag-and-drop, Shell context menu, Lixeira real, ACLs, UNC/SMB, volumes removíveis, SmartScreen, acessibilidade ou assinatura Authenticode. A matriz de compatibilidade continua explícita sobre essas lacunas.

## Reprodução local

```text
cargo fmt --all -- --check
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --target x86_64-pc-windows-gnu --all-targets --all-features -- -D warnings
cargo check --target x86_64-pc-windows-gnu --all-targets --all-features
cargo build --release --target x86_64-pc-windows-gnu
```

A execução nativa completa ocorrerá automaticamente no job `Rust quality (windows-latest)` do `.github/workflows/ci.yml`.
