# Rovex

O Rovex é um explorador de arquivos local, seguro e leve para Windows 10 e 11, escrito prioritariamente em Rust. O projeto evolui incrementalmente: cada recurso precisa ser real, testável e documentado antes de ser considerado concluído.

> O estado atual é um protótipo desktop funcional de navegação e operações locais. Ele abre uma janela Slint, lista diretórios reais, navega para pastas, copia/move/renomeia/exclui dentro dos limites de segurança e exibe erros controlados, mas ainda não é um Explorer completo nem uma release de distribuição.

## Estado atual

O núcleo implementa listagem real de diretórios, classificação de arquivos, diretórios e links simbólicos sem seguir o destino automaticamente, normalização de destinos, erros estruturados, criação de diretório, renomeação, exclusão limitada a arquivos, links e diretórios vazios e cópia atômica sem sobrescrita. A interface Slint executa o carregamento e as operações em workers limitados, atualiza o modelo no thread principal, descarta resultados obsoletos de navegações concorrentes e encerra workers cooperativamente. A modal exige confirmação explícita, mostra progresso/cancelamento e recarrega a pasta após o resultado. A barra lateral mostra somente locais conhecidos que existem, sem análise de espaço ou varredura de unidades. O histórico mantém pilhas reais de voltar/avançar, e a seleção local suporta clique, Ctrl-clique, Shift-clique e Ctrl+A. O filtro atual atua somente sobre a pasta carregada, usa uma fila latest-only com um worker dedicado e trabalha sobre snapshots compartilhados, sem pesquisa recursiva nem thread por tecla. O visual usa tokens Slint locais para cores, espaçamento, raios e estados, sem blur, animações contínuas ou dependências extras.

| Área | Estado |
|---|---|
| Núcleo Rust | Implementado e testado |
| Listagem real de diretório | Implementada |
| Cópia sem sobrescrita por padrão | Implementada e testada |
| Operações visuais de copiar/mover/renomear/excluir | Implementadas com confirmação, worker, progresso e cancelamento |
| Criação e renomeação | Implementadas e testadas |
| Exclusão segura limitada | Implementada para arquivos, links e diretórios vazios |
| Janela desktop Slint | Implementada |
| Barra de endereço e pasta pai | Implementadas |
| Lista visual, atualização e ativação de diretórios | Implementadas |
| Workers e descarte de resultados obsoletos | Implementados |
| Filtro local sem varredura global | Implementado com fila limitada |
| Histórico voltar/avançar | Implementado e testado |
| Seleção múltipla local | Implementada e testada com clique, Ctrl/Shift e Ctrl+A |
| Barra lateral com locais existentes | Implementada sem análise de discos |
| Worker único de carregamento e encerramento cooperativo | Implementado e validado com stress/smoke |
| Design System local de tokens | Implementado e validado visualmente |
| Pesquisa global, abas, thumbnails e drag and drop | Planejados |
| Conversores multimídia/PDF/OCR | Fora da primeira fatia; não simulados |
| Instalador, assinatura e atualização | Planejados para a fase de distribuição |

## Verificação local

O toolchain de desenvolvimento está fixado em `rust-toolchain.toml`. Execute:

```text
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
cargo audit
cargo deny check
```

O modo desktop abre a janela com um diretório inicial opcional:

```text
cargo run -- .
```

O modo CLI permanece disponível para ambientes sem display e para diagnóstico headless:

```text
cargo run -- --cli .
```

A execução gráfica foi testada em display virtual com carregamento de `/tmp`, interação com a barra de endereço, filtro local, seleção múltipla, histórico de navegação e captura de screenshots. A validação cruzada gera um PE32+ x86-64 para Windows; a execução efetiva em Windows 10/11, incluindo DPI, permissões Win32, junctions, UNC/SMB e acessibilidade nativa, ainda precisa ocorrer em runners ou máquinas Windows.

## Documentação

A decisão arquitetural está em [`docs/architecture.md`](docs/architecture.md), o plano incremental está em [`docs/implementation-plan.md`](docs/implementation-plan.md), a estratégia de testes está em [`docs/testing.md`](docs/testing.md), a pesquisa do Slint está em [`docs/slint-research.md`](docs/slint-research.md), a compatibilidade de plataforma está em [`COMPATIBILITY.md`](COMPATIBILITY.md), a matriz de dependências está em [`DEPENDENCIES.md`](DEPENDENCIES.md), o relatório desta modernização está em [`MODERNIZATION_REPORT.md`](MODERNIZATION_REPORT.md) e as notas das fontes anteriores estão em [`docs_research_notes.md`](docs_research_notes.md).

## Segurança e dependências

O Rovex não executa arquivos durante a navegação, não usa shell para operações de arquivo e não envia conteúdo local para serviços externos. A UI não executa filesystem no thread visual. O filtro local não varre subpastas e seu worker processa no máximo a consulta pendente mais recente. Destinos são normalizados antes de operações sensíveis, resultados atrasados de workers são descartados por geração e as operações visuais somente começam após confirmação explícita.

A política `cargo-deny` permite somente as licenças observadas e revisadas na árvore atual, incluindo as referências customizadas declaradas pelo Slint. `cargo audit` não encontrou vulnerabilidades, mas reporta quatro advisories de manutenção transitivos do toolkit: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. Não há upgrade seguro informado pela base RustSec para essa cadeia; eles permanecem visíveis como warnings e estão registrados em [`docs/slint-research.md`](docs/slint-research.md).

## Licença

Este projeto é distribuído sob a licença MIT. A dependência Slint possui sua própria expressão de licenciamento e deve permanecer coberta pelas referências oficiais do crate antes de qualquer distribuição comercial. Dependências e backends futuros deverão ser auditados quanto a manutenção, vulnerabilidades e compatibilidade de licença antes de serem adicionados.
