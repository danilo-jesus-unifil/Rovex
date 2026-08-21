# Relatório de refatoração arquitetural do Rovex

**Data:** 17 de agosto de 2026
**Projeto:** Rovex: explorador de arquivos nativo para Windows 10/11
**Autor:** Manus AI
**HEAD validado:** `eca6051`
**Branch:** `main`

## 1. Resultado

A refatoração preservou o comportamento funcional e deixou todos os arquivos de produção em `src/` e `ui/` abaixo de 400 linhas. A extração separou operações de arquivo, estado de navegação e seleção, schedulers, conversores, handlers da janela, tokens visuais, controles, barras de ferramentas, modelos Slint e overlays.

A interface mantém o tema escuro, os botões de navegação e atualização, abas, listagem, menu contextual com quatro conversões e diálogo de operação. A validação gráfica confirmou inicialização, fluxo de abas e conversão JPEG XL com o binário e a imagem em pastas diferentes.

## 2. Comparação objetiva de tamanho

O baseline utilizado foi o checkpoint `4e2ccdf`, criado antes da modularização completa. A contagem abaixo considera arquivos Rust e Slint de produção, excluindo artefatos de `target/`.

| Arquivo no baseline | Linhas antes | Resultado após a refatoração |
|---|---:|---|
| `src/converters.rs` | 1.542 | Substituído por 9 módulos; maior módulo atual: `windows_backend.rs` com 298 linhas |
| `src/desktop.rs` | 911 | Fachada de 31 linhas; handlers separados em 8 módulos |
| `src/desktop/jobs.rs` | 888 | Substituído por 8 módulos; maior módulo atual: `operations.rs` com 208 linhas |
| `src/desktop/state.rs` | 663 | Substituído por 6 módulos; maior módulo atual: `tests.rs` com 249 linhas |
| `src/operations.rs` | 531 | Substituído por 5 módulos; maior módulo atual: `copy.rs` com 200 linhas |
| `ui/main.slint` | 848 | Fachada principal com 317 linhas; UI distribuída em 6 arquivos |

No estado final, o maior arquivo de produção é `src/security.rs`, com 367 linhas. Nenhum arquivo ultrapassa 400 linhas.

## 3. Estrutura final por responsabilidade

| Área | Organização final | Responsabilidade preservada |
|---|---|---|
| Operações | `src/operations/{error,copy,entry,tests,mod}.rs` | Validação, cópia atômica, criação, exclusão, renomeação e testes |
| Estado desktop | `src/desktop/state/{models,navigation,listing,view,tests,mod}.rs` | Modelos, histórico de abas, listagem, seleção e view-model |
| Jobs desktop | `src/desktop/jobs/{types,operations,operation_scheduler,conversion,conversion_scheduler,filter_scheduler,load_scheduler,mod}.rs` | Tipos de requisição, execução assíncrona, cancelamento e atualização da UI |
| Conversores | `src/converters/{types,paths,process_output,backend,windows_backend,process,pipeline,tests,mod}.rs` | Resolução de backends, fallbacks, processos FFmpeg/ffprobe e publicação segura |
| Handlers | `src/desktop/handlers/{navigation,selection,operations,confirmation,conversions,dialogs,filter,lifecycle,mod}.rs` | Registro dos callbacks da `MainWindow` e coordenação dos fluxos da UI |
| Contexto | `src/desktop/context.rs` | Agregação explícita dos modelos, schedulers, seleção, abas e `Weak<MainWindow>` |
| UI Slint | `design_tokens.slint`, `components.slint`, `data.slint`, `toolbars.slint`, `overlays.slint`, `main.slint` | Tema escuro, controles reutilizáveis, barras, listagem, menu contextual e diálogo |

A divisão da UI segue o sistema de módulos do Slint: tipos exportados podem ser importados por outros arquivos `.slint`, e componentes exportados podem ser compostos em um componente principal.[1] A fachada `main.slint` continua exportando `MainWindow`, `LocationRow`, `FileRow`, `TabRow` e `DesignTokens`, mantendo a superfície usada pelo código Rust.

> “Similarly, components exported from other files may be imported.”: documentação oficial do Slint sobre módulos.[1]

O `build.rs` também foi atualizado com `cargo:rerun-if-changed` para cada módulo Slint. Assim, uma alteração em tokens, controles, modelos, toolbar ou overlays força a recompilação apropriada da interface, em vez de depender somente da data de `main.slint`.

## 4. Correção de compatibilidade Windows encontrada na validação

O check cruzado revelou que `windows_backend.rs` ainda importava helpers como se eles estivessem diretamente na fachada `converters`, embora, após a modularização, eles estivessem em `converters::backend`. A correção passou a importar explicitamente esses helpers de `super::backend` e incluiu `Command` e `Stdio` no escopo do backend Windows. O alvo `x86_64-pc-windows-gnu` foi recompilado com sucesso depois dessa correção.

Essa verificação foi importante porque o caminho Linux não exercita o módulo condicionado por `cfg(windows)`. A compilação cruzada foi mantida como parte da validação final justamente para capturar esse tipo de regressão de plataforma.

## 5. Validação executada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 44 aprovados, 0 falhos, 2 ignorados explicitamente |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado sem warnings |
| `cargo check --target x86_64-pc-windows-gnu` | Aprovado |
| `cargo build --release` | Aprovado; binário otimizado gerado |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok` |
| `git diff --check` | Aprovado, sem erros de whitespace |
| Inventário de arquivos `src/` e `ui/` | Nenhum arquivo acima de 400 linhas |
| `scripts/smoke_gui.sh` | Processo permaneceu ativo até o timeout esperado |
| `scripts/capture_tabs.sh` | Abertura, alternância e fechamento de aba aprovados |
| `scripts/test_ui_jxl_conversion.sh` | Saída JPEG XL criada com 67 bytes |

Os dois testes ignorados continuam explicitamente documentados: o benchmark manual de filtro de 100 mil itens e o teste de conversões reais que requer FFmpeg e ffprobe no ambiente de teste.

## 6. Auditoria de dependências

O `cargo audit` terminou com sucesso e reportou quatro avisos de dependências não mantidas, sem registrar neste ciclo um advisory de vulnerabilidade explorável. Os avisos pertencem à árvore resolvida de dependências e não foram introduzidos pela divisão de módulos do Rovex.

| Crate | Versão | Advisory | Natureza |
|---|---:|---|---|
| `bincode` | 2.0.1 | [RUSTSEC-2025-0141][2] | Não mantida |
| `paste` | 1.0.15 | [RUSTSEC-2024-0436][3] | Não mantida |
| `rustybuzz` | 0.20.1 | [RUSTSEC-2026-0206][4] | Não mantida |
| `ttf-parser` | 0.25.1 | [RUSTSEC-2026-0192][5] | Não mantida |

O `cargo deny check` aprovou advisories, bans, licenças e fontes. Como os crates estão associados à árvore transitiva da stack gráfica e não houve falha de segurança acionável no conjunto avaliado, não foi feita uma atualização cega que pudesse quebrar Slint 1.17.1 ou a compatibilidade Windows. A recomendação é reavaliar esses avisos quando uma versão compatível da cadeia Slint substituir os crates, especialmente antes de uma atualização de dependências.

## 7. Pontos de recuperação e commits

O estado estável anterior à divisão da UI está preservado na branch `backup/before-ui-modularization-2026-08-17`. Também permanecem disponíveis os checkpoints históricos anteriores, incluindo `backup/before-full-modularization-2026-08-17`.

| Commit | Conteúdo |
|---|---|
| `ab104bf` | Divisão inicial das operações de arquivo em módulos |
| `6e675d7` | Modularização de handlers desktop, estado compartilhado, jobs e conversores |
| `eca6051` | Modularização da interface Slint e correção dos imports do backend Windows |

O working tree está limpo. A branch `main` está quatro commits à frente de `origin/main`; nenhum push foi forçado ou executado durante esta etapa.

## 8. Estado validado

A regra de modularização foi cumprida para Rust e Slint, os limites de responsabilidade estão separados, a API pública da janela foi preservada, a compilação Linux e Windows GNU foi validada, as verificações estáticas passaram e os fluxos gráficos essenciais foram exercitados sob Xvfb.

### Referências

[1]: https://docs.slint.dev/latest/docs/slint/guide/language/coding/file/ "Slint Docs: The .slint File e Modules"
[2]: https://rustsec.org/advisories/RUSTSEC-2025-0141 "RustSec: Bincode is unmaintained"
[3]: https://rustsec.org/advisories/RUSTSEC-2024-0436 "RustSec: paste is no longer maintained"
[4]: https://rustsec.org/advisories/RUSTSEC-2026-0206 "RustSec: rustybuzz is unmaintained"
[5]: https://rustsec.org/advisories/RUSTSEC-2026-0192 "RustSec: ttf-parser is unmaintained"
