# Rovex v0.1.9 — relatório técnico da release

**Autor:** Manus AI  
**Data:** 17 de agosto de 2026  
**Repositório:** [danilo-jesus-unifil/Rovex](https://github.com/danilo-jesus-unifil/Rovex)

## Resumo

A v0.1.9 conclui a refatoração arquitetural do Rovex sem remover ou alterar os fluxos funcionais da v0.1.8. O explorador continua nativo em Rust com Slint 1.17.1, tema escuro, abas reais, navegação, listagem local, seleção, operações de arquivo, menu contextual e conversões FFmpeg/ffprobe.

A regra objetiva da etapa foi atendida: nenhum arquivo de produção Rust ou Slint em `src/` e `ui/` ultrapassa 400 linhas. A modularização usa fronteiras naturais, sem traits, factories, wrappers ou crates adicionais criados apenas para reduzir contagem de linhas.

> O binário release foi executado sob Xvfb; o smoke test de abas passou e a conversão JPEG XL pela UI criou uma saída de 67 bytes fora da pasta do binário.

## Mudanças implementadas

| Área | Implementação |
|---|---|
| Operações | `src/operations.rs` foi dividido em `error.rs`, `copy.rs`, `entry.rs`, `tests.rs` e `mod.rs`. |
| Estado desktop | `src/desktop/state.rs` foi dividido em modelos, navegação, listagem, view-model, testes e fachada. |
| Jobs | `src/desktop/jobs.rs` foi dividido em tipos, operações, schedulers de operação/conversão/filtro/carregamento e fachadas. |
| Conversores | `src/converters.rs` foi dividido em tipos, caminhos, saída de processos, backends, backend Windows, processo, pipeline, testes e fachada. |
| Handlers | `src/desktop.rs` ficou como composition root; callbacks foram separados em módulos de navegação, seleção, operações, confirmação, conversões, diálogos, filtro e lifecycle. |
| UI Slint | `main.slint` foi reduzido a 317 linhas; tokens, controles, modelos, toolbar e overlays foram extraídos para módulos importados. |
| Build | `build.rs` observa todos os arquivos Slint importados com `cargo:rerun-if-changed`. |
| Windows | Corrigidos imports de helpers em `windows_backend.rs`, além de `Command` e `Stdio`, permitindo o check cruzado Windows GNU. |
| Documentação | `CHANGELOG.md`, `../reference/known-issues.md` e este relatório foram atualizados para v0.1.9. |

A divisão da UI utiliza imports e componentes exportados do Slint, conforme a documentação oficial de módulos.[1] A API gerada de `MainWindow`, `LocationRow`, `FileRow` e `TabRow` foi preservada para o código Rust.

## Comparação de tamanho

O baseline arquitetural foi `4e2ccdf`, anterior à modularização completa.

| Arquivo anterior | Linhas antes | Organização atual |
|---|---:|---|
| `src/converters.rs` | 1.542 | 9 módulos; maior módulo: 298 linhas |
| `src/desktop.rs` | 911 | Fachada de 31 linhas e handlers separados |
| `src/desktop/jobs.rs` | 888 | 8 módulos; maior módulo: 208 linhas |
| `src/desktop/state.rs` | 663 | 6 módulos; maior módulo: 249 linhas |
| `src/operations.rs` | 531 | 5 módulos; maior módulo: 200 linhas |
| `ui/main.slint` | 848 | Fachada de 317 linhas e 5 módulos auxiliares |

No estado final, o maior arquivo de produção é `src/security.rs`, com 367 linhas. Os arquivos Slint finais têm 20, 53, 118, 183, 256 e 317 linhas, respectivamente.

## Validação de qualidade

A sequência abaixo foi executada no estado v0.1.9 antes do empacotamento:

```text
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo check --target x86_64-pc-windows-gnu --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --target x86_64-pc-windows-gnu --all-targets --all-features -- -D warnings
cargo audit
cargo deny check
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
```

| Verificação | Resultado |
|---|---|
| Formatação | Passou sem diferenças |
| Check Linux | Passou em todos os alvos e features |
| Check Windows GNU | Passou em `x86_64-pc-windows-gnu` |
| Testes | 44 aprovados, 0 falhos, 2 ignorados explicitamente |
| Clippy | Passou sem warnings, inclusive no alvo Windows GNU |
| Auditoria de políticas | `cargo deny check`: advisories, bans, licenses e sources aprovados |
| Build release | Linux e Windows GNU concluídos |

Os dois testes ignorados continuam intencionais: benchmark manual de filtro e teste de conversões reais que requer backend FFmpeg/ffprobe instalado. Os fluxos reais de conversão e os smoke tests gráficos foram executados separadamente.

## Evidências funcionais

| Fluxo | Resultado |
|---|---|
| `scripts/smoke_gui.sh` | O processo release permaneceu ativo até o timeout esperado sob Xvfb. |
| `scripts/capture_tabs.sh` | Abriu segunda aba, alternou para a primeira e fechou a segunda. |
| `scripts/test_ui_jxl_conversion.sh` | Menu contextual e diálogo produziram saída JPEG XL de 67 bytes. |
| Diretório separado | A validação foi executada com o binário separado da pasta da imagem, preservando a descoberta de backends. |
| Limite arquitetural | Nenhum arquivo Rust ou Slint de produção excedeu 400 linhas. |

## Auditoria de dependências

O `cargo audit` concluiu sem advisory de vulnerabilidade explorável, mas registrou quatro avisos de crates não mantidos na árvore transitiva resolvida: `bincode` 2.0.1, `paste` 1.0.15, `rustybuzz` 0.20.1 e `ttf-parser` 0.25.1. Os avisos permanecem documentados; não foi feita atualização cega capaz de quebrar Slint 1.17.1 ou a compatibilidade Windows.

O `cargo deny check` aprovou todas as categorias configuradas. Esses avisos devem ser reavaliados quando uma cadeia compatível do Slint substituir os componentes transitivos, antes de uma atualização de dependências.

## Artefatos publicados

A release segue o padrão v0.1.8 e contém os seguintes assets:

| Asset | Conteúdo |
|---|---|
| `rovex-v0.1.9-linux-x86_64.tar.gz` | Binário Linux release, README, CHANGELOG, relatório, PNG do ícone e desktop entry. |
| `rovex-v0.1.9-windows-x86_64.zip` | Executável Windows release, README, CHANGELOG, relatório, ICO e PNG do ícone. |
| `SHA256SUMS.txt` | SHA-256 dos dois pacotes distribuíveis. |
| `./RELEASE_REPORT-v0.1.9.md` | Este relatório técnico. |

Os pacotes são portáveis e não baixam executáveis em runtime. A conversão continua dependendo de `ffmpeg` e `ffprobe` disponíveis no sistema ou nos caminhos seguros de descoberta documentados.

## Limitações conhecidas

O check cruzado e o empacotamento Windows GNU não substituem a execução manual em Windows 10/11 real. Ainda devem ser validados em uma máquina Windows o DPI, a renderização nativa do ICO, acessibilidade nativa, junctions, caminhos UNC/SMB, paths longos, permissões, arquivos em uso, manifesto PE, instalador, assinatura digital e desinstalação.

## Referências

[1]: https://docs.slint.dev/latest/docs/slint/guide/language/coding/file/ "Slint Docs — The .slint File e Modules"
[2]: https://rustsec.org/advisories/RUSTSEC-2025-0141 "RustSec — Bincode is unmaintained"
[3]: https://rustsec.org/advisories/RUSTSEC-2024-0436 "RustSec — paste is no longer maintained"
[4]: https://rustsec.org/advisories/RUSTSEC-2026-0206 "RustSec — rustybuzz is unmaintained"
[5]: https://rustsec.org/advisories/RUSTSEC-2026-0192 "RustSec — ttf-parser is unmaintained"
