# Relatório de implementação e hardening — Rovex

## Resultado atual

O Rovex foi revisado de forma incremental sobre a fundação Rust existente. Foram criados backups remotos antes das alterações, os testes foram ampliados, uma falha real de normalização de destinos foi encontrada e corrigida, o toolchain foi atualizado para Rust 1.97.1 e a bateria de qualidade passou novamente.

O branch principal está sincronizado com o repositório remoto. O projeto continua sendo uma **fundação funcional de desenvolvimento**, não um Explorer desktop completo: a interface Slint, abas, pesquisa, thumbnails, conversores, instalador e integração com o shell ainda não foram implementados e permanecem explicitamente documentados como pendências.

## Backups criados

| Branch | Ponto protegido |
|---|---|
| [`backup/pre-hardening-2026-08-14`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/pre-hardening-2026-08-14) | Estado anterior ao ciclo de hardening |
| [`backup/stable-d431ba7`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/stable-d431ba7) | Commit estável anterior às correções |

Ambos foram publicados no GitHub antes das alterações. O commit que contém o hardening está no branch `main` e será identificado no momento da publicação desta etapa.

## Correções realizadas

A validação de destinos agora normaliza o diretório pai por canonicalização, rejeita raiz e componentes finais ambíguos, compara origem e destino depois da normalização e mantém a recusa de sobrescrita por padrão. Isso corrige o caso em que caminhos equivalentes, como `nested/../arquivo`, poderiam ser tratados como diferentes por uma comparação textual simples.

A exclusão continua deliberadamente limitada a arquivos, links e diretórios vazios. Um teste de regressão confirma que um diretório não vazio produz erro controlado e permanece intacto. A listagem passou a ter teste específico para links simbólicos, verificando que o link é identificado sem seguir seu destino.

Os diretórios temporários dos testes deixaram de depender apenas de nanossegundos do relógio e passaram a usar contador atômico combinado com PID, reduzindo a possibilidade de colisões em testes concorrentes. A política `cargo-deny` foi reduzida às licenças realmente presentes no estado atual, eliminando warnings de permissões não utilizadas.

O toolchain foi fixado em `rust-toolchain.toml` com Rust 1.97.1, rustfmt, Clippy e alvo Windows x64. A CI foi alinhada à mesma versão para evitar divergência entre verificação local e pipeline.

## Verificações concluídas

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo test --all-targets --all-features` | **12 testes aprovados, 0 falhas** |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo build --release` | Aprovado |
| `cargo audit` 0.22.2 | Aprovado; nenhum advisory para as dependências atuais |
| `cargo deny check` 0.20.2 | Aprovado; advisories, bans, licenças e fontes OK |
| `cargo check --target x86_64-pc-windows-gnu --all-targets --all-features` | Aprovado |
| `cargo build --release --target x86_64-pc-windows-gnu` | Aprovado |
| Execução do binário Linux | Aprovada; listagem real do repositório |
| Artefato Windows | Validado como PE32+ x86-64 |

O build Windows foi realizado com MinGW no ambiente Linux. Isso confirma compilação e formato do artefato, mas não substitui a execução em Windows 10/11, testes de DPI, acessibilidade, permissões Win32, junctions, UNC/SMB, instalador e desinstalador.

## Auditoria manual

A busca por `unsafe`, `TODO`, `FIXME`, `panic!`, `unwrap` e `expect` encontrou apenas `expect` em auxiliares de teste. Não há `unsafe` nem caminhos de produção dependentes de `panic`. O crate não possui dependências de runtime além da biblioteca padrão; `Cargo.lock` permanece auditável.

## Próxima etapa

A próxima etapa técnica deve conectar o núcleo a uma interface desktop real, mantendo os contratos atuais. A UI deverá navegar por diretórios reais, executar comandos fora do thread visual, exibir erros estruturados e declarar acessibilidade. A validação em Windows 10 e 11 precisa ocorrer antes de anunciar compatibilidade final.

## Referências técnicas

A escolha provisória do Slint permanece registrada porque o toolkit se apresenta como uma solução declarativa para Rust e desktop compilado [1]. A documentação do Slint recomenda declarar papel, rótulo e ações de acessibilidade em componentes personalizados [2]. A Microsoft recomenda definir DPI awareness no manifesto do processo, com `<dpiAwareness>` e fallback quando necessário [3].

[1]: https://slint.dev/ "Slint — página oficial"
[2]: https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/ "Slint Docs — Best Practices"
[3]: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process "Microsoft Learn — Setting the default DPI awareness for a process"
