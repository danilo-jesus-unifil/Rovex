# Rovex v0.1.11 — Release report

## Resumo

A versão `v0.1.11` entrega os lotes de Preview, busca recursiva, configurações persistentes, Lixeira do Windows e a primeira distribuição portable verificável. O objetivo deste release é fornecer um artefato Windows x86-64 extraível, sem instalador, sem serviço e sem download de executáveis em runtime.

| Item | Valor |
|---|---|
| Versão | `0.1.11` |
| Target | `x86_64-pc-windows-gnu` |
| Perfil | `release`, LTO thin, um codegen unit, símbolos removidos, panic abort |
| Artefato | `rovex-v0.1.11-windows-x86_64-portable.zip` |
| Verificação | `rovex-v0.1.11-windows-x86_64-portable.sha256` e `scripts/verify_windows_portable.sh` |
| Assinatura | **Não assinado**; nenhum certificado foi inventado ou embutido |
| FFmpeg | Opcional, externo e nunca baixado em runtime |

## Conteúdo e segurança

O ZIP contém `rovex.exe`, `LICENSE`, `README.md`, `COMPATIBILITY.md`, `PORTABLE.txt` e `DISTRIBUTION-MANIFEST.txt`. O manifesto informa target, commit, perfil, ausência de assinatura e ausência de downloads em runtime. O verificador calcula SHA-256, rejeita entradas absolutas ou com traversal, extrai em diretório temporário, exige os arquivos obrigatórios e reinspeciona o manifesto PE Windows.

No Windows, exclusões confirmadas usam `SHFileOperationW` com `FOF_ALLOWUNDO` e preservam diretórios não vazios. Se o Shell falhar, o item permanece no filesystem; não há fallback permanente silencioso. Configurações ficam no perfil do usuário e não no diretório portable.

## Validação executada

A matriz do lote foi executada com `cargo fmt --all -- --check`, `cargo test --all-targets --all-features`, Clippy com `-D warnings`, `cargo audit`, `cargo deny check advisories licenses bans sources`, `cargo check`/`cargo build --release` para Windows GNU, verificação do manifesto PE e todos os smoke tests gráficos existentes, incluindo Settings. O pacote foi gerado duas vezes em diretórios separados; ZIP e checksum foram byte a byte idênticos. O verificador passou e confirmou o manifesto embutido.

A execução nativa em Windows 10/11, SmartScreen, certificado Authenticode, ACLs, DPI, UNC/SMB, volumes removíveis, paths longos e restauração real da Lixeira continuam gates de compatibilidade. Essas limitações estão registradas em [`COMPATIBILITY.md`](COMPATIBILITY.md) e não são tratadas como “prontas” apenas por cross-build.

## Procedimento de publicação

O asset deve ser anexado a uma release GitHub com o arquivo `.sha256` ao lado. O usuário deve verificar a soma publicada antes de extrair. Como o artefato não é assinado, o Windows pode exibir SmartScreen; a decisão de execução deve considerar a origem do download e a soma verificada.

As instruções para produzir e verificar novamente são:

```text
scripts/package_windows_portable.sh x86_64-pc-windows-gnu dist
scripts/verify_windows_portable.sh dist/rovex-v0.1.11-windows-x86_64-portable.zip
```
