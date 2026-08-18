# Rovex v0.1.10 — Relatório técnico da release

**Data:** 18 de agosto de 2026  
**Commit de preparação:** `fdb2b0a` + sincronização do lockfile `89f090e`  
**Escopo:** publicação otimizada da etapa Foundation do issue #2.

## Resumo

A v0.1.10 é uma release portable que publica a primeira etapa do master prompt do issue #2: auditoria Foundation, plano incremental e documentação reconciliada. Nenhuma funcionalidade nova foi simulada ou adicionada sem contrato e testes. O comportamento funcional da v0.1.9 foi preservado.

O pacote continua oferecendo listagem real, navegação, histórico, abas, seleção múltipla, filtro local, sidebar, operações controladas de copiar/mover/renomear/excluir, menu contextual, quatro conversões locais e tratamento estruturado de erros. A interface permanece baseada em Slint 1.17.1 com tema escuro e o binário release usa LTO thin, um codegen unit, strip de símbolos e `panic = "abort"`.

## Mudanças documentais e de engenharia

A release inclui `ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md`, cobrindo arquitetura, módulos, funcionalidades existentes e ausentes, dívida técnica, riscos de segurança e desempenho, Windows 10/11, UI/UX, acessibilidade, testes, documentação, dependências e ordem recomendada.

Também inclui `docs/issue-2-execution-plan.md`, que divide o master prompt em Foundation, Core Explorer, Search, Preview, Advanced Tools, Windows Integration e Distribution, com critérios de aceite e workflow de commits pequenos. O README foi atualizado para indicar v0.1.10 como release portable mais recente e para distinguir tags públicas de refinamentos posteriores na branch principal. `Cargo.toml` e `Cargo.lock` foram sincronizados para 0.1.10.

## Validação executada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 44 aprovados, 0 falhos, 2 ignorados explicitamente |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo audit` | Sem vulnerabilidades bloqueantes; warnings transitivos de manutenção documentados |
| `cargo deny check` | Aprovado em advisories, bans, licenças e fontes |
| Check Windows GNU | Aprovado |
| Clippy Windows GNU | Aprovado |
| Build release Linux x86_64 | Aprovado |
| Build release Windows x86_64 GNU | Aprovado |
| Smoke GUI | Aprovado |
| Fluxo de abas | Aprovado |
| Menu contextual e conversões | Aprovados |
| JPEG XL em diretórios separados | Aprovado; codec confirmado como `jpegxl` |

## Limitações conhecidas

Esta release não declara compatibilidade nativa completa com Windows 10/11. A execução nativa identificada, DPI por monitor, leitor de tela, alto contraste, paths longos, UNC/SMB, junctions/reparse points, arquivos em uso, manifesto efetivamente incorporado, instalador, assinatura e atualização continuam gates futuros documentados na auditoria e na matriz de compatibilidade.

A cadeia do Slint mantém quatro warnings transitivos de manutenção (`bincode`, `paste`, `rustybuzz` e `ttf-parser`) sem advisory de vulnerabilidade bloqueante identificado nesta verificação. Nenhum executável externo é baixado em runtime.

## Artefatos

A release publica os pacotes portable Linux e Windows, `SHA256SUMS.txt` e este relatório. Os checksums são gerados somente dos dois pacotes finais, publicados no asset separado e verificados após download da release publicada. O relatório não incorpora os próprios hashes para evitar uma dependência circular entre o conteúdo do relatório e o checksum do pacote.

## Reprodutibilidade

O build Linux usa `cargo build --release`. O build Windows usa `cargo build --release --target x86_64-pc-windows-gnu`. A distribuição não contém arquivos temporários, diretórios de build ou segredos.
