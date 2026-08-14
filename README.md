# Rovex

O Rovex é a base de um explorador de arquivos local, seguro e leve para Windows 10 e 11, escrito prioritariamente em Rust. O projeto segue uma evolução incremental: cada recurso precisa ser real, testável e documentado antes de ser considerado concluído.

> O estado atual é uma fundação de desenvolvimento. A interface desktop ainda não foi incorporada; portanto, o repositório não deve ser tratado como um Explorer final nem como uma demonstração visual pronta.

## Estado atual

A primeira fatia implementa um núcleo compilável com listagem real de diretórios, classificação de arquivos, diretórios e links simbólicos sem seguir o destino automaticamente, normalização de destinos, erros estruturados, criação de diretório, renomeação, exclusão limitada a arquivos, links e diretórios vazios, além de cópia atômica para um destino que não exista. A cópia utiliza arquivo temporário, sincronização, validação do tamanho e publicação por renomeação; o arquivo original permanece preservado.

| Área | Estado |
|---|---|
| Núcleo Rust | Implementado |
| Listagem real de diretório | Implementada |
| Cópia sem sobrescrita por padrão | Implementada e testada |
| Criação e renomeação | Implementadas e testadas |
| Exclusão segura limitada | Implementada para arquivos, links e diretórios vazios |
| Interface desktop | Próxima etapa |
| Pesquisa, abas e thumbnails | Planejadas |
| Conversores multimídia/PDF/OCR | Fora da primeira fatia; não simulados |
| Instalador e assinatura | Planejados para a fase de distribuição |

## Verificação local

O toolchain de desenvolvimento está fixado em `rust-toolchain.toml`. Execute:

```text
cargo fmt --all -- --check
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo audit
cargo deny check
```

O núcleo também foi verificado com build release cruzado para Windows x64:

```text
cargo build --release --target x86_64-pc-windows-gnu
```

O binário de desenvolvimento lista um diretório real e pode ser executado com:

```text
cargo run -- .
```

## Documentação

A decisão arquitetural inicial está em [`docs/architecture.md`](docs/architecture.md), o plano incremental está em [`docs/implementation-plan.md`](docs/implementation-plan.md) e as notas das fontes consultadas estão em [`docs_research_notes.md`](docs_research_notes.md).

## Segurança

O Rovex não executa arquivos durante a navegação, não usa shell para operações de arquivo, não baixa codecs arbitrários e não deve exigir administrador para tarefas comuns. As operações de maior risco serão isoladas em workers somente quando houver backend e política de limites definidos. As limitações atuais, incluindo a exclusão sem recursão, são deliberadas para evitar que uma API incompleta produza exclusões perigosas.

## Licença

Este projeto é distribuído sob a licença MIT. Dependências e backends futuros deverão ser auditados quanto a manutenção, vulnerabilidades e compatibilidade de licença antes de serem adicionados.
