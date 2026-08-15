# Rovex v0.1.0 — Release Report

**Pacote:** `rovex` 0.1.0  
**Commit de código que gerou os binários:** `e7b285c`  
**Tag planejada:** `v0.1.0`  
**Data:** 15 de agosto de 2026  
**Target Linux:** `x86_64-unknown-linux-gnu`  
**Target Windows:** `x86_64-pc-windows-gnu`

> Estado do relatório: os checks, builds, testes de artefato e hashes foram executados. A tag e a publicação final somente devem ocorrer depois que este relatório, a lista de hashes e o working tree estiverem commitados e o CI da ponta final estiver aprovado.

## Toolchain e versão

| Item | Resultado |
|---|---|
| Rust | **PASS** — `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Cargo | **PASS** — `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Toolchain fixada | **PASS** — `1.97.1-x86_64-unknown-linux-gnu`, com `rustfmt`, `clippy` e target Windows GNU |
| Edition | **PASS** — Rust 2024 |
| MSRV declarado | **PASS** — `rust-version = "1.97"`, alinhado ao toolchain e ao Slint resolvido |
| Versão Rovex | **PASS** — `0.1.0` |

## Checks automatizados

| Verificação | Estado | Evidência |
|---|---|---|
| `cargo fmt --all -- --check` | **PASS** | Executado no commit `e7b285c` |
| `cargo check` | **PASS** | Executado no commit `e7b285c` |
| `cargo check --all-targets` | **PASS** | Executado no commit `e7b285c` |
| `cargo check --all-features` | **PASS** | Executado no commit `e7b285c` |
| `cargo check --all-targets --all-features` | **PASS** | Executado no commit `e7b285c` |
| `cargo test` | **PASS** | 32 aprovados, 0 falhas, 1 benchmark ignorado |
| `cargo test --all-targets --all-features` | **PASS** | 32 aprovados, 0 falhas, 1 benchmark ignorado |
| `cargo clippy --all-targets --all-features -- -D warnings` | **PASS** | Nenhum warning bloqueador |
| `cargo doc --all-features --no-deps` | **PASS** | Documentação gerada |
| `cargo audit` | **PASS** | Nenhuma vulnerabilidade explorável; avisos transitivos de manutenção permanecem visíveis |
| `cargo deny check` | **PASS** | Advisories, bans, licenças e fontes aprovados |
| `cargo tree` | **PASS** | Árvore revisada; sem dependência direta inesperada |

## Builds e artefatos

| Artefato | Estado | Tamanho |
|---|---|---:|
| `target/release/rovex` | **PASS** — build release Linux | 16.333.200 bytes |
| `target/x86_64-pc-windows-gnu/release/rovex.exe` | **PASS** — cross-build release Windows GNU | 12.679.680 bytes |
| PE subsystem | **PASS** — `Windows GUI`, versão de subsystem 5.2 | Verificado com `x86_64-w64-mingw32-objdump` |
| Pacote Linux `.tar.gz` | **PASS** — conteúdo listado e extração testada | 7.658.413 bytes |
| Pacote Windows `.zip` | **PASS** — `unzip -t` aprovado | 6.344.253 bytes |
| Artefato Linux extraído | **PASS** — CLI e GUI executados em ambiente limpo | Xvfb 1100×720 |
| Artefato Windows nativo | **NOT APPLICABLE** | Runner Windows executa build/teste; execução manual Windows 10/11 não ocorreu nesta sessão |

## Testes manuais e integrados

| Teste | Estado | Observação |
|---|---|---|
| Startup do release Linux | **PASS** | Binário extraído de pacote iniciou sob Xvfb |
| Listagem real | **PASS** | Arquivo temporário `example.txt` apareceu na GUI e no CLI |
| Copiar/mover/renomear/excluir | **PASS** | Smoke real verificou o filesystem após cada operação |
| Clique simples/double-click/Enter/Escape | **PASS** | Smoke X11 reproduzível e capturas revisadas |
| Navegação rápida latest-only | **PASS** | 12 pastas; a última navegação venceu resultados obsoletos |
| Filtro latest-only | **PASS** | `file-12-12` retornou exatamente `file-12-12.txt` |
| Encerramento controlado | **PASS** | Processo release terminou sem worker residual observado |
| Windows 10 22H2 nativo | **NOT APPLICABLE** | Não há execução nativa Windows disponível nesta sessão |
| Windows 11 nativo | **NOT APPLICABLE** | Não há execução nativa identificada nesta sessão |
| Instalador/desinstalador | **NOT APPLICABLE** | O Rovex ainda distribui pacote portable; não existe instalador no repositório |
| Assinatura digital | **NOT APPLICABLE** | Artefatos não assinados nesta etapa |
| Atualizador/rollback de instalação | **NOT APPLICABLE** | Atualizador ainda não implementado |

## Problemas encontrados e corrigidos durante a preparação

A auditoria de artefatos detectou que o PE Windows era originalmente emitido como `CUI`, o que poderia abrir um console indesejado ao iniciar a interface. O entrypoint agora aplica `windows_subsystem = "windows"` somente em builds Windows release; o perfil debug mantém o console para o modo CLI de diagnóstico. O PE foi reconstruído e verificado como `Windows GUI`.

Também foi corrigida a declaração de `rust-version`, que estava em 1.92 apesar de o toolchain fixado e as dependências verificadas exigirem a linha Rust 1.97. Nenhuma dependência direta ou transitiva foi alterada nesta preparação de release.

## Dependências e vulnerabilidades

O runtime continua usando Slint 1.17.1 com renderer software, acessibilidade, `compat-1-2` e backend Winit por plataforma. O lockfile permanece consistente e os avisos transitivos de manutenção (`bincode`, `paste`, `rustybuzz` e `ttf-parser`) seguem documentados; `cargo audit` não indicou vulnerabilidade explorável e `cargo deny check` passou.

## Hashes SHA-256

Os hashes completos estão em [`SHA256SUMS-v0.1.0.txt`](SHA256SUMS-v0.1.0.txt). Os pacotes principais são:

| Arquivo | SHA-256 |
|---|---|
| `Rovex-v0.1.0-linux-x64.tar.gz` | `8891c9c5d4da3dddca96e96003014605ea894fc3864fd5d31f107ffa498938c7` |
| `Rovex-v0.1.0-windows-x64.zip` | `f8ff3478bf658607dd409e9ce84af1fda96292638382c8671edbd46d92af45b2` |
| `Rovex-v0.1.0-linux-x64` | `8804ff760a08dfbfa4e0d061b28d3002832bb37f37f46188d65553c3a14e57be` |
| `Rovex-v0.1.0-windows-x64.exe` | `6ae4babddd1def01df34061b427fde6cbc37a5cb08491ab1405fb643797abb0b` |

## Problemas ainda conhecidos

A release é portable e não inclui instalador, assinatura ou atualização automática. A execução nativa identificada em Windows 10/11, DPI, leitor de tela nativo, paths longos, UNC/SMB, reparse points, USB, permissões Windows e arquivo bloqueado ainda não foi realizada nesta sessão. Também permanecem fora do escopo atual abas, split view, pesquisa global, preview, thumbnails, drag and drop, clipboard, conversores e undo/redo. A release não deve ser descrita como certificada para esses cenários.

## Referências

[1]: FINAL_STABILITY_REPORT.md "Relatório final de estabilidade"
[2]: COMPATIBILITY.md "Matriz de compatibilidade"
[3]: DEPENDENCIES.md "Inventário de dependências"
[4]: SECURITY.md "Política de segurança"
