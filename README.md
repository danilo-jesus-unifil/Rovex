# Rovex

O Rovex é um explorador de arquivos local, seguro e leve para Windows 10 e 11, escrito prioritariamente em Rust. O projeto evolui incrementalmente: cada recurso precisa ser real, testável e documentado antes de ser considerado concluído.

> O estado atual é a release portable `v0.1.28` de um protótipo desktop funcional. Ele abre uma janela Slint com tema escuro, abas reais e ícone próprio, lista diretórios reais, navega para pastas, copia/move/renomeia/exclui dentro dos limites de segurança, persiste preferências por usuário, envia exclusões à Lixeira no Windows, oferece busca recursiva, preview seguro de imagens/texto, drag-and-drop e conversões reais via FFmpeg/ffprobe. A distribuição é um ZIP portable sem assinatura digital; ainda não é um Explorer completo, não possui instalador MSI/MSIX nem atualização automática.

## Estado atual

O núcleo implementa listagem real de diretórios, classificação de arquivos e diretórios e links simbólicos sem seguir o destino automaticamente, normalização de destinos, erros estruturados, criação de diretório, renomeação, exclusão limitada a arquivos, links e diretórios vazios e cópia atômica sem sobrescrita. Conversões externas configuram grupos de processos Unix ou Job Objects Windows para que cancelamento e timeout terminem descendentes do FFmpeg/ffprobe antes de aguardar seus pipes. A interface Slint executa o carregamento, as operações e as conversões em workers limitados, atualiza o modelo no thread principal, descarta resultados obsoletos de navegações concorrentes e encerra workers cooperativamente. A modal exige confirmação explícita, mostra progresso/cancelamento e recarrega a pasta após o resultado. A barra lateral mostra somente locais conhecidos que existem, sem análise de espaço ou varredura de unidades. O histórico mantém pilhas reais de voltar/avançar, e a seleção local suporta clique, Ctrl-clique, Shift-clique e Ctrl+A. O filtro atual atua somente sobre a pasta carregada, usa uma fila latest-only com um worker dedicado e trabalha sobre snapshots compartilhados, sem pesquisa recursiva nem thread por tecla. O visual usa tokens Slint locais para tema escuro, cores, espaçamento, raios e estados, sem blur, animações contínuas ou dependências extras.

| Área | Estado |
|---|---|
| Núcleo Rust | Implementado e testado |
| Listagem real de diretório | Implementada |
| Cópia sem sobrescrita por padrão | Implementada e testada |
| Operações visuais de copiar/mover/renomear/excluir | Implementadas com confirmação, worker, progresso e cancelamento |
| Criação e renomeação | Implementadas e testadas |
| Exclusão segura/Lixeira | Implementada para arquivos, links e diretórios vazios; Lixeira no Windows e remoção permanente apenas no fallback Unix de desenvolvimento |
| Janela desktop Slint | Implementada |
| Barra de endereço e pasta pai | Implementadas |
| Lista visual, atualização e ativação de diretórios e arquivos | Implementadas; arquivos usam o aplicativo padrão no Windows |
| Workers e descarte de resultados obsoletos | Implementados |
| Filtro local sem varredura global | Implementado com fila limitada |
| Histórico voltar/avançar | Implementado e testado por aba |
| Abas de navegação | Implementadas com histórico independente, abertura, seleção e fechamento |
| Seleção múltipla local | Implementada e testada com clique, Ctrl/Shift e Ctrl+A |
| Barra lateral com locais existentes | Implementada sem análise de discos |
| Worker único de carregamento e encerramento cooperativo | Implementado e validado com stress/smoke |
| Design System local de tokens e tema escuro | Refatorado e validado visualmente com superfícies, raios e variantes semânticas |
| Ícone do aplicativo | PNG/ICO próprio, embutido no Windows e disponível para desktop entry Linux |
| Menu contextual por clique direito | Implementado com ações de arquivo, Terminal, Open With explícito e conversões condicionais |
| JPEG/PNG e imagens compatíveis → JPEG XL | Implementado via FFmpeg/libjxl e validado por ffprobe |
| WAV/MP3/FLAC e áudio compatível → Opus | Implementado via FFmpeg/libopus e validado por ffprobe |
| Imagens → PNG e áudio → FLAC | Implementado via FFmpeg e validado por ffprobe |
| Busca recursiva, preview e drag and drop | Implementados com workers, limites e cancelamento; execução nativa Windows ainda pendente |
| Abrir Terminal aqui | Implementado com worker e cascata Windows Terminal → PowerShell → cmd; desabilitado no Linux |
| Abrir | Implementado com `ShellExecuteExW`, verbo padrão, worker COM STA sem DDE assíncrono, erros tipados e validação contra reparse points; desabilitado no Linux |
| Reparse points | Listagem, navegação, busca recursiva, pais de destinos e exclusão recusam ou tratam junctions e demais reparse points sem seguir o destino |
| Nomes reservados Windows | `CON`/`PRN`/`AUX`/`NUL`, `COM1`–`COM9`, `LPT1`–`LPT9` e equivalentes sobrescritos ¹/²/³ são rejeitados, inclusive com extensões |
| Abrir com... | Implementado via diálogo nativo `SHOpenWithDialog` para arquivo regular; continua separado de Abrir e desabilitado no Linux |
| Conversores PDF/OCR | Fora do escopo desta release; não simulados |
| Contenção de processos externos | Implementada com grupos Unix, Job Objects Windows e cancelamento da árvore |
| Reserva de temporários de conversão | Atômica com `create_new`, sem janela `exists`→uso |
| Reserva e uso de temporário | Placeholder reservado atomicamente e preservado até o spawn do FFmpeg |
| Pacote portable Windows | Implementado em ZIP v0.1.28 com manifesto e SHA-256 |
| Instalador, assinatura e atualização | Ainda planejados; não há certificado ou assinatura fictícia |

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
python3 scripts/verify_markdown_layout.py
python3 -m unittest scripts/test_verify_markdown_layout.py -v
./scripts/audit_edge_cases.sh
./scripts/package_windows_portable.sh
./scripts/verify_windows_portable.sh dist/rovex-v0.1.28-windows-x86_64-portable.zip
./scripts/test_verify_windows_portable.sh dist/rovex-v0.1.28-windows-x86_64-portable.zip
./scripts/test_activation_contract.sh
./scripts/test_windows_native_contract.sh
./scripts/test_reserved_windows_names_contract.sh
./scripts/test_ffmpeg_discovery_contract.sh
./scripts/test_process_containment_contract.sh
./scripts/test_ui_jxl_separate_dirs.sh
./scripts/capture_tabs.sh
```

O modo desktop abre a janela com um diretório inicial opcional:

```text
cargo run -- .
```

O modo CLI permanece disponível para ambientes sem display e para diagnóstico headless:

```text
cargo run -- --cli .
```

A execução gráfica foi testada em display virtual com carregamento de `/tmp`, interação com a barra de endereço, filtro local, seleção múltipla, histórico de navegação, clique direito, confirmação de conversão e captura de screenshots. A validação cruzada gera um PE32+ x86-64 para Windows; a execução efetiva em Windows 10/11, incluindo DPI, permissões Win32, junctions, UNC/SMB e acessibilidade nativa, ainda precisa ocorrer em runners ou máquinas Windows. As conversões exigem `ffmpeg.exe` e `ffprobe.exe`. O Rovex tenta overrides absolutos, `PATH`, PATH persistente do usuário e do sistema no Windows, App Paths, o diretório do próprio executável, o diretório explicitamente adjacente ao FFmpeg, variáveis `FFMPEG_HOME`/`FFMPEG_ROOT`/`FFMPEG_DIR`/`FFMPEG_PATH`, WinGet, Chocolatey, Scoop e outros diretórios conhecidos com profundidade e quantidade limitadas. O CWD não é adicionado implicitamente; `SearchPathW` e `where.exe` não são usados como fallbacks porque suas buscas padrão podem reintroduzir o CWD. Overrides que apontam para uma pasta também são tratados como raízes que podem conter `ffmpeg.exe`/`ffprobe.exe`. Para diagnóstico controlado, também aceita `ROVEX_FFMPEG_PATH` e `ROVEX_FFPROBE_PATH` com caminhos absolutos. O Rovex não baixa executáveis em runtime.

## Documentação

A decisão arquitetural está em [`docs/architecture.md`](./docs/architecture.md), o plano incremental está em [`./docs/plans/implementation-plan.md`](./docs/plans/implementation-plan.md), a estratégia de testes está em [`./docs/reference/testing.md`](./docs/reference/testing.md), a pesquisa do Slint está em [`./docs/research/slint-research.md`](./docs/research/slint-research.md), a compatibilidade de plataforma está em [`COMPATIBILITY.md`](./COMPATIBILITY.md), a matriz de dependências está em [`./docs/reference/DEPENDENCIES.md`](./docs/reference/DEPENDENCIES.md), o relatório desta modernização está em [`./docs/reports/MODERNIZATION_REPORT.md`](./docs/reports/MODERNIZATION_REPORT.md), o relatório final de estabilidade está em [`./docs/reports/FINAL_STABILITY_REPORT.md`](./docs/reports/FINAL_STABILITY_REPORT.md), o histórico de versões está em [`CHANGELOG.md`](./CHANGELOG.md) e o relatório desta release está em [`./docs/releases/RELEASE_REPORT-v0.1.28.md`](./docs/releases/RELEASE_REPORT-v0.1.28.md), a pesquisa de distribuição está em [`./docs/research/distribution-research-2026-08-19.md`](./docs/research/distribution-research-2026-08-19.md), a pesquisa de processos externos está em [`./docs/research/external-process-research-2026-08-19.md`](./docs/research/external-process-research-2026-08-19.md), a auditoria visual está em [`./docs/audits/ui-audit-initial.md`](./docs/audits/ui-audit-initial.md), o plano visual está em [`./docs/plans/ui-refactor-plan.md`](./docs/plans/ui-refactor-plan.md), o teste de abas está em [`scripts/capture_tabs.sh`](./scripts/capture_tabs.sh), o teste gráfico de diretórios separados está em [`scripts/test_ui_jxl_separate_dirs.sh`](./scripts/test_ui_jxl_separate_dirs.sh) e a pesquisa de descoberta de backends está em [`./docs/research/ffmpeg-discovery-research.md`](./docs/research/ffmpeg-discovery-research.md), a pesquisa de contenção de processos está em [`./docs/research/process-containment-research-2026-08-20.md`](./docs/research/process-containment-research-2026-08-20.md), e a pesquisa da ação Terminal está em [`./docs/research/terminal-research-2026-08-19.md`](./docs/research/terminal-research-2026-08-19.md) e a pesquisa de Open With está em [`./docs/research/open-with-research-2026-08-19.md`](./docs/research/open-with-research-2026-08-19.md), enquanto o contrato de ativação está em [`./docs/research/activation-research-2026-08-20.md`](./docs/research/activation-research-2026-08-20.md) e a pesquisa de erros de ativação está em [`./docs/research/activation-errors-research-2026-08-20.md`](./docs/research/activation-errors-research-2026-08-20.md), e a validação de caminhos longos está em [`./docs/research/long-path-validation-2026-08-20.md`](./docs/research/long-path-validation-2026-08-20.md), e a pesquisa de reparse points está em [`./docs/research/reparse-point-classification-2026-08-20.md`](./docs/research/reparse-point-classification-2026-08-20.md), enquanto a pesquisa de nomes reservados está em [`./docs/research/reserved-device-names-2026-08-20.md`](./docs/research/reserved-device-names-2026-08-20.md).

## Segurança e dependências

O Rovex não executa arquivos durante a simples navegação; a ativação só ocorre por duplo clique, Enter ou ação explícita Abrir para um arquivo regular validado. Open With e Terminal permanecem ações separadas. O Rovex não usa shell para operações de arquivo e não envia conteúdo local para serviços externos. A UI não executa filesystem no thread visual. O filtro local não varre subpastas e seu worker processa no máximo a consulta pendente mais recente. Destinos são normalizados antes de operações sensíveis, resultados atrasados de workers são descartados por geração e as operações visuais somente começam após confirmação explícita.

A política `cargo-deny` permite somente as licenças observadas e revisadas na árvore atual, incluindo as referências customizadas declaradas pelo Slint. `cargo audit` não encontrou advisories de vulnerabilidade bloqueantes, mas reporta quatro warnings transitivos de manutenção do toolkit: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. Eles permanecem visíveis como warnings, são permitidos pela configuração atual e estão registrados em [`./docs/research/slint-research.md`](./docs/research/slint-research.md).

## Licença

Este projeto é distribuído sob a licença MIT. A dependência Slint possui sua própria expressão de licenciamento e deve permanecer coberta pelas referências oficiais do crate antes de qualquer distribuição comercial. Dependências e backends futuros deverão ser auditados quanto a manutenção, vulnerabilidades e compatibilidade de licença antes de serem adicionados.
