# Rovex — Modernization Report

**Data:** 15 de agosto de 2026  
**Baseline:** commit `1433e28`  
**Checkpoint:** `backup/before-modernization-2026-08-15`  
**Estado do relatório:** modernização local e validação CI concluídas; o run `31864602133` aprovou auditoria, Linux, Windows e cross-build Windows GNU.

## Resumo executivo

A modernização não foi tratada como um `cargo update` cego. A auditoria verificou o Rust estável disponível, o registry, as features do Slint, a árvore transitiva, o build script, o CI, chamadas sensíveis e o target Windows. O Rust 1.97.1 já era a release estável atual e já estava fixado; portanto, não houve uma troca artificial de toolchain. A mudança estrutural segura foi migrar a crate para **edition 2024**, corrigir os dois lints que o Clippy moderno passou a exigir nesse contexto e atualizar somente os dois crates transitivos que o Cargo identificou como compatíveis.

A versão direta do Slint permaneceu em **1.17.1**, que foi confirmada pelo registry e pelas fontes oficiais como a release adequada disponível durante a execução [1] [2]. A decisão preserva o renderer de software, acessibilidade, compatibilidade e a redução Linux X11-only obtida na etapa anterior.

## Estado antes e depois

| Item | Antes | Depois | Decisão |
|---|---|---|---|
| Rust toolchain | 1.97.1 | 1.97.1 | Já era a release estável atual; mantida |
| Edição Rust | 2021 | 2024 | Atualizada após experimento isolado e validação completa |
| `rust-version` | 1.92 | 1.92 | Mantida como MSRV compatível com Slint 1.17.1 |
| Slint runtime | 1.17.1 | 1.17.1 | Mantido; nenhuma versão posterior real foi encontrada |
| Slint build | 1.17.1 | 1.17.1 | Mantido alinhado ao runtime |
| `uuid` transitivo | 1.24.0 | 1.24.1 | Atualização compatível do `cargo update` |
| `wayland-backend` transitivo | 0.3.16 | 0.3.17 | Atualização compatível do `cargo update` |
| Dependências diretas removidas | Nenhuma | Nenhuma | O workspace já tinha somente Slint e Slint-build |
| Dependências diretas substituídas | Nenhuma | Nenhuma | Não havia alternativa comprovadamente melhor |
| CI | Testes, Clippy e build nativo | + `cargo check` e cross-build Windows GNU | Lacunas de compilação e target cobertas |

## APIs e código

O inventário não encontrou blocos `unsafe` fora das integrações FFI Win32 já existentes e documentadas, nem `deprecated`, `unwrap` ou `expect` em caminhos de produção; os `expect` encontrados pertencem a fixtures de testes. O Clippy com `-D warnings` revelou dois `collapsible_if` no experimento edition 2024. Eles foram corrigidos com let-chains idiomáticos, sem `allow` ou compatibilidade artificial com código antigo.

O `build.rs` continua mínimo e reproduzível: observa `ui/main.slint`, chama `slint_build::compile` e não baixa arquivos, executa shell ou depende de uma máquina específica. Não foram adicionados bindings Windows redundantes, logger, serializador, conversor, codec externo ou sistema de configuração.

## Features e segurança

A configuração mantém `default-features = false`. Permanecem habilitados `renderer-software`, `accessibility` e `compat-1-2`, porque são requisitos funcionais ou de segurança do projeto. O backend Linux usa `backend-winit-x11` no ambiente CI/Xvfb; Windows usa `backend-winit`. A documentação desktop oficial do Slint lista Windows 10 x86-64 e Windows 11 x86-64/aarch64 entre suas plataformas testadas [2].

A alteração não introduziu APIs Windows exclusivas do Windows 11. O núcleo usa APIs portáveis de filesystem e mantém validações contra traversal, raiz, componentes pai simbólicos e publicação sem sobrescrita. O renderer de software continua sendo o fallback para máquinas sem GPU dedicada.

## Métricas antes/depois

| Métrica | Baseline `1433e28` | Modernizado | Variação |
|---|---:|---:|---:|
| Pacotes resolvidos | 587 | 587 | 0 |
| Nomes duplicados | 44 | 44 | 0 |
| `Cargo.lock` | 150.741 bytes | 150.741 bytes | 0 bytes |
| Linux release | 16.328.464 bytes | 16.329.232 bytes | +768 bytes |
| Windows GNU release | 12.675.584 bytes | 12.676.096 bytes | +512 bytes |
| Clean build Linux | 132,101 s | 133,288 s | +1,187 s |
| Pico RSS clean build | 945.484 KiB | 945.648 KiB | +164 KiB |

Os builds clean foram medidos em cópias separadas com o mesmo toolchain, script e máquina. A pequena variação de tamanho e tempo não é apresentada como ganho de desempenho. A atualização foi aceita por modernização e manutenção, sem regressão grosseira mensurável.

## Auditoria de segurança e supply chain

`cargo audit` terminou sem vulnerabilidades exploráveis. A resolução continua emitindo quatro avisos de manutenção transitivos da cadeia Slint: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. Eles não foram ignorados automaticamente; a decisão e a ausência de upgrade seguro aplicável permanecem documentadas em `docs/slint-research.md`.

`cargo deny check` passou em advisories, bans, licenças e fontes. A política permite somente registries e licenças observados, incluindo as referências customizadas do Slint. Nenhum download em build, binário externo ou binding Windows novo foi adicionado.

## Verificação executada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 31 aprovados, 0 falhas, 1 ignorado |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo audit` | Aprovado, com quatro avisos transitivos de manutenção documentados |
| `cargo deny check` | Aprovado |
| `cargo build --release` | Aprovado |
| `cargo build --release --target x86_64-pc-windows-gnu` | Aprovado |
| Smoke Xvfb de navegação e UI | Aprovado antes e depois das alterações de dependência |
| Smoke de operações reais | Copiar, renomear e excluir aprovados no filesystem |
| CI Linux/Windows/cross | Run `31864602133` aprovado: auditoria, Linux, Windows e Windows GNU cross-build |

## Compatibilidade Windows

O alvo de produto continua Windows 10/11 x64. O target GNU cross foi compilado localmente e o job Windows hospedado já exercita teste/build nativo. Ainda não houve, nesta sessão, execução manual identificada especificamente como Windows 10 22H2/build 19045 nem matriz separada de Windows 11. Por isso, DPI por monitor, alto contraste, leitor de tela, tema, UNC/SMB, volumes removíveis, reparse points, paths longos, arquivos em uso e manifesto PE permanecem pendências honestas.

A política de compatibilidade está detalhada em `COMPATIBILITY.md`. Em particular, o projeto ainda não possui manifesto Windows embutido que prove `asInvoker`, DPI awareness e long-path awareness; isso é um gate de distribuição futuro, não uma propriedade declarada sem evidência.

## Commits e rollback

A modernização foi dividida em commits pequenos:

| Commit | Conteúdo |
|---|---|
| `37a2f46` | Migração edition 2024 e correções idiomáticas do Clippy |
| `82d6380` | Refresh transitivo, Cargo.lock e reforço do CI |
| `backup/before-modernization-2026-08-15` | Estado estável anterior para rollback |

Se uma atualização futura quebrar uma plataforma, o rollback deve ser seletivo: primeiro comparar a árvore e reverter apenas o grupo responsável, sem abandonar a edição 2024 ou os testes de CI sem reproduzir a causa.

## Problemas ainda conhecidos

A stack direta não possui uma release posterior verificada à adequada. Os avisos de manutenção transitivos permanecem dependentes do upstream Slint. A execução nativa com Windows 10/11, o manifesto PE, o teste de DPI e a matriz de filesystem Windows ainda precisam de ambiente Windows identificado. Conversores, codecs, thumbnails, pesquisa global, abas e outros recursos não existem no escopo e não foram simulados.

> **Conclusão:** a stack está modernizada dentro do que foi possível verificar com segurança: Rust 1.97.1 atual, edition 2024, Slint 1.17.1 mantido, transitivas compatíveis atualizadas, CI ampliado e sem vulnerabilidades conhecidas. A conformidade nativa completa do Windows permanece um gate separado e explicitamente não falsificado.

## Referências

[1]: https://crates.io/crates/slint "slint — crates.io"
[2]: https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/ "Desktop — Slint Documentation"
[3]: https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/ "Announcing Rust 1.97.1 — Rust Blog"
[4]: https://slint.dev/blog/slint-1.17-released "Slint 1.17 Released — Slint Blog"
[5]: https://learn.microsoft.com/en-us/windows/release-health/release-information "Windows release information — Microsoft Learn"
