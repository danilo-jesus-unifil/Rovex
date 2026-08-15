# Rovex — Dependency Inventory

**Data da auditoria:** 15 de agosto de 2026. Este inventário separa dependências diretas, toolchain e transitivas. O princípio adotado foi usar a versão estável adequada realmente disponível, não substituir crates apenas para aumentar números de versão.

## Dependências diretas

O Rovex possui dependências diretas de runtime para a UI e para a integração Windows, além de uma dependência direta de build. O binding Windows é compilado somente no alvo Windows e permanece fora da árvore Linux.

| Dependência | Versão resolvida | Uso | Features efetivas | Decisão | Compatibilidade |
|---|---:|---|---|---|---|
| `slint` | 1.17.1 | Janela desktop, componentes Slint, event loop, modelo visual e acessibilidade | Linux: `backend-winit-x11`, `renderer-software`, `accessibility`, `compat-1-2`; Windows/Unix não-Linux: `backend-winit`, `renderer-software`, `accessibility`, `compat-1-2`; `default-features = false` | **Manter pin exato** | MSRV publicada 1.92; documentação desktop lista Windows 10 x86-64 e Windows 11 x86-64/aarch64 [1] |
| `slint-build` | 1.17.1 | Compilação de `ui/main.slint` no `build.rs` | Defaults do crate | **Manter pin exato e alinhado ao runtime** | Mesma release do `slint`; evita compiler/runtime divergentes |
| `windows-sys` | 0.61.2 | `SHGetKnownFolderPath` para locais conhecidos do Windows | `Win32_Foundation`, `Win32_System_Com`, `Win32_UI_Shell`; somente Windows | **Manter restrito ao target** | API Win32 compatível com Windows 10/11; sem efeito em Linux |

A consulta do registry realizada durante a auditoria retornou `slint = 1.17.1`, `slint-build = 1.17.1` e `windows-sys = 0.61.2` como versões usadas e verificadas para esta etapa. `windows-sys` foi adicionado somente para o target Windows e não altera o backend Linux.

## Toolchain e build

| Item | Estado modernizado | Motivo |
|---|---|---|
| Rust | 1.97.1 | Release estável atual verificada na fonte oficial; inclui correção de miscompilação LLVM [2] |
| Edição | Rust 2024 | Migração validada com check, testes e Clippy estrito; permite sintaxe moderna e elimina compatibilidade artificial com edição 2021 |
| `rust-version` | 1.92 | MSRV declarada compatível com Slint 1.17.1; não foi elevada sem necessidade |
| Componentes | `rustfmt`, `clippy` | Verificação local e CI |
| Target cross | `x86_64-pc-windows-gnu` | Artefato PE32+ Windows x64 reproduzível no ambiente Linux |
| Build script | `slint-build::compile("ui/main.slint")` | Sem downloads, shell ou comportamento dependente de uma máquina |

## Atualização transitiva controlada

O `cargo update --dry-run` indicou apenas duas atualizações compatíveis e não alterou os crates diretos: `uuid` 1.24.0 → 1.24.1 e `wayland-backend` 0.3.16 → 0.3.17. A atualização real foi aplicada, seguida de check, testes, Clippy, auditoria de advisories, cargo-deny e builds Linux/Windows.

| Métrica | Antes da modernização | Depois | Interpretação |
|---|---:|---:|---|
| Pacotes resolvidos | 587 | 587 | Nenhum crescimento da resolução |
| Nomes duplicados | 44 | 44 | Duplicações são transitivas do toolkit e não controláveis pelo manifesto do Rovex |
| `Cargo.lock` | 150.741 bytes | 150.741 bytes | O refresh trocou checksums/versões sem alterar o tamanho medido |
| Árvore `cargo metadata` | 587 nós resolvidos | 587 nós resolvidos | Sem crates diretos redundantes |
| Linux release | 16.328.464 bytes | 16.329.232 bytes | +768 bytes; variação pequena, sem ganho artificial declarado |
| Windows GNU release | 12.675.584 bytes | 12.676.096 bytes | +512 bytes; compatibilidade preservada |
| Clean build Linux | 132,101 s | 133,288 s | +1,187 s, dentro da variação de máquina; RSS praticamente estável |
| Pico RSS do clean build | 945.484 KiB | 945.648 KiB | +164 KiB |

Os números de build foram obtidos com o mesmo medidor parametrizado, em cópias do commit baseline `1433e28` e do estado modernizado. Eles não são uma promessa de desempenho universal; servem para detectar regressões grosseiras.

## Features e consolidação

A análise confirmou que `default-features = false` é necessário. `renderer-software` não foi removido porque é o renderer sem GPU dedicado exigido pelo objetivo de compatibilidade. `accessibility` não foi removido porque é requisito do PROMPTMASTER. `compat-1-2` permanece obrigatório pelo Slint 1.17.1. O backend Linux foi restringido a X11 porque é o ambiente realmente exercitado pelo CI/Xvfb; Windows mantém Winit nativo.

As 44 duplicações de nomes incluem famílias como `windows-sys`, `calloop`, `thiserror`, `syn`, `tiny-skia`, `rustix` e bindings de plataformas não alvo. A árvore mostra que são introduzidas por diferentes partes transitivas do Slint; remover uma versão manualmente quebraria a resolução ou uma feature de plataforma. Nenhuma dependência transitiva foi forçada para uma versão incompatível.

## Licenças e advisories

O projeto é MIT. O Slint publica uma expressão que inclui `GPL-3.0-only` ou as referências royalty-free/software próprias; a política `deny.toml` permite as referências customizadas observadas, além das licenças SPDX necessárias na árvore. `cargo deny check` passou após o refresh.

`cargo audit` não encontrou vulnerabilidades exploráveis. Permanecem quatro avisos de manutenção transitivos na cadeia do Slint — `bincode`, `paste`, `rustybuzz` e `ttf-parser` — sem atualização segura indicada pela resolução atual. Eles não foram escondidos por exceções; estão documentados em `docs/slint-research.md` e continuam visíveis no CI.

## Alternativas avaliadas

Não foi introduzida outra biblioteca de UI, renderer, binding Windows, logger, serializador, conversor ou sistema de configuração. O projeto não usa essas categorias diretamente. Substituir Slint por outro toolkit ou adicionar bindings Windows seria uma mudança arquitetural, não uma consolidação segura de dependências, e não foi justificado por nenhuma incompatibilidade encontrada.

> **Decisão:** a stack direta mantém Rust 1.97.1 e Slint 1.17.1; `windows-sys` foi adicionado de modo restrito ao target Windows para consultar Known Folders oficiais, sem alterar o alvo Linux. A árvore continua auditada por cargo-audit e cargo-deny.

## Referências

[1]: https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/ "Desktop — Slint Documentation"
[2]: https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/ "Announcing Rust 1.97.1 — Rust Blog"
[3]: https://crates.io/crates/slint "slint — crates.io"
[4]: https://slint.dev/blog/slint-1.17-released "Slint 1.17 Released — Slint Blog"
