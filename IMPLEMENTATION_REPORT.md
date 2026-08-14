# Relatório de implementação — Rovex

## Resultado

O prompt master foi analisado e convertido em uma primeira fundação Rust real para o repositório `danilo-jesus-unifil/Rovex`. O projeto não ficou limitado a uma tela demonstrativa: o núcleo agora lista diretórios reais, classifica entradas sem seguir links automaticamente, executa operações básicas de arquivo e trata falhas por tipos estruturados.

A implementação foi publicada no branch `main` no commit [`e8d4a2a`](https://github.com/danilo-jesus-unifil/Rovex/commit/e8d4a2a), cujo estado remoto foi confirmado após o push.

## Entregas

| Área | Resultado |
|---|---|
| Manifesto Cargo | Crate Rust `rovex`, licença MIT e perfil release otimizado |
| Filesystem | Listagem real, metadados por `symlink_metadata`, diretórios, arquivos e links |
| Segurança | Recusa de raiz, validação de origem/destino e prevenção de sobrescrita por padrão |
| Operações | Cópia atômica com temporário, criação, renomeação e exclusão limitada |
| Erros | Tipos estruturados para filesystem, validação e operações |
| Documentação | Arquitetura, compatibilidade, testes, plano incremental, limitações e segurança |
| CI | Workflow Linux/Windows com fmt, test, Clippy, build e auditoria de dependências |
| Política de dependências | `deny.toml` com advisories, licenças e fontes restritas |

A exclusão recursiva não foi implementada de propósito. A primeira fatia só remove arquivos, links e diretórios vazios; isso reduz o risco de transformar uma API incompleta em uma operação destrutiva ampla. Conversores, OCR, thumbnails, abas, pesquisa, instalador e integração silenciosa com o shell continuam fora do escopo imediato e não foram simulados.

## Verificações executadas

A cadeia final executada no ambiente foi:

```text
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo run -- .
```

O resultado foi **sete testes aprovados, zero falhas**, Clippy sem diagnósticos com avisos tratados como erros, build release concluído e execução do binário com listagem real do repositório. `cargo audit` e `cargo deny` foram configurados na CI, mas não foram executados localmente porque os executáveis não estavam instalados no ambiente; esse ponto permanece como critério de aceite da pipeline.

## Decisões técnicas

A escolha provisória para a interface é Slint, por sua integração com Rust e modelo de UI compilada [1]. A documentação do framework recomenda declarar papel, rótulo e ações de acessibilidade nos componentes personalizados [2]. O documento arquitetural registra essa escolha como provisória até que o protótipo seja validado em Windows 10 com teclado, leitor de tela, DPI e múltiplos monitores.

A configuração de DPI deverá ser feita no manifesto do processo. A documentação da Microsoft recomenda o manifesto e descreve `<dpiAwareness>` com fallback para versões compatíveis, evitando depender de configuração tardia por API [3]. A compatibilidade efetiva com Windows 10/11 ainda não foi testada neste ambiente Linux e permanece explicitamente pendente.

## Próximo passo recomendado

A próxima etapa deve implementar a camada de aplicação e um protótipo desktop funcional que conecte a listagem do núcleo a uma interface Slint. O protótipo deverá navegar por um diretório real, exibir erros estruturados e manter operações fora do thread da UI. Só depois de validar essa fundação em Windows 10/11 será prudente adicionar pesquisa, abas, thumbnails e conversores isolados.

## Arquivos principais

- [`src/filesystem.rs`](src/filesystem.rs): listagem e metadados.
- [`src/security.rs`](src/security.rs): política de destinos e validações.
- [`src/operations.rs`](src/operations.rs): cópia atômica e operações básicas.
- [`docs/architecture.md`](docs/architecture.md): decisões e camadas.
- [`docs/implementation-plan.md`](docs/implementation-plan.md): roadmap incremental.
- [`SECURITY.md`](SECURITY.md): modelo de ameaça e política inicial.
- [`.github/workflows/ci.yml`](.github/workflows/ci.yml): pipeline de qualidade.

## Referências

[1]: https://slint.dev/ "Slint — página oficial"
[2]: https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/ "Slint Docs — Best Practices"
[3]: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process "Microsoft Learn — Setting the default DPI awareness for a process"
