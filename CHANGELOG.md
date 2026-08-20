# Changelog

Todas as mudanças relevantes do Rovex são registradas neste arquivo.

## [0.1.15] — 2026-08-19

A versão 0.1.15 adiciona a ação contextual `Abrir Terminal aqui` com cascata Windows segura e rejeição de symlinks/reparse points no diretório alvo.

| Área | Mudança |
|---|---|
| Menu contextual | Novo botão escuro e acessível para abrir o terminal no diretório da pasta selecionada ou no pai de um arquivo. |
| Windows | Tentativas em ordem `wt.exe`, PowerShell e Prompt de Comando, com `--startingDirectory`, `current_dir` e argumentos separados; nenhum shell concatenado é usado. |
| Concorrência | A abertura é executada em worker nomeado e o resultado é publicado no event loop Slint, sem bloquear a UI nem esperar o terminal fechar. |
| Segurança | Caminhos relativos, ausentes, inválidos e diretórios symlink/reparse point são recusados; no Linux o botão fica desabilitado. |
| UI | Menu contextual rolável e responsivo preserva tema escuro, bordas arredondadas, estados disabled e todos os conversores existentes. |
| Testes | 94 testes host passaram; cross-check, Clippy host/cross, link de testes Windows, smoke gráfico e CI `windows-latest` passaram. |
| Documentação | Contrato, fallback, fontes oficiais e limites de validação interativa foram registrados em `./docs/research/terminal-research-2026-08-19.md`, README e COMPATIBILITY. |

## [0.1.14] — 2026-08-19

A versão 0.1.14 evolui a integração da Lixeira no Windows para `IFileOperation`/COM sem remover a rota de compatibilidade pré-execução.

| Área | Mudança |
|---|---|
| Windows Shell | Exclusão usa `SHCreateItemFromParsingName`, `IShellItem` e `FOFX_RECYCLEONDELETE` dentro do worker de operações; não há shell externo nem execução na UI. |
| Fallback | `SHFileOperationW` permanece somente para indisponibilidade de COM, parsing ou enfileiramento antes de `PerformOperations`; falhas após o início não são repetidas. |
| Erros | HRESULTs negativos e `GetAnyOperationsAborted` são convertidos em `OperationError` estruturado; exclusão permanente silenciosa continua proibida. |
| Segurança | Mantidos validação de origem, bloqueio de raiz, rejeição de diretórios não vazios, `FOFX_EARLYFAILURE` e `FOF_NORECURSION`. |
| Testes | Cobertura de buffers UTF-16, GUIDs oficiais, flags, HRESULT e regra de fallback; suíte nativa Windows passou no CI. |
| Qualidade | Host/cross Clippy, check, link de testes Windows, audit/deny, builds release, manifesto e todos os smoke tests passaram. |

## [0.1.13] — 2026-08-19

A versão 0.1.13 corrige três falhas reais encontradas pelo primeiro runner Windows nativo do Rovex.

| Área | Correção |
|---|---|
| Filesystem Windows | `validate_destination` rejeita `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `COM1`–`COM9` e `LPT1`–`LPT9`, incluindo variantes com extensão e pontuação/espaços finais. |
| Settings Windows | O round-trip usa caminho absoluto nativo; gravações concorrentes são serializadas pelo processo para evitar `PermissionDenied` durante `MoveFileExW`. |
| CI | O primeiro workflow Windows revelou e documentou as regressões; o smoke CLI nativo preserva Unicode e espaços sem abrir a UI. |
| Qualidade | Correções passam em testes host, Clippy/cross-build Windows e verificação de manifesto; a execução do workflow confirma o comportamento nativo. |

## [0.1.12] — 2026-08-19

A versão 0.1.12 endurece a execução de conversores externos e corrige uma regressão de validação que usava uma opção incompatível com o ffprobe disponível.

| Área | Mudança |
|---|---|
| Processos | Cleanup centralizado com `kill` + `wait` + join dos leitores em cancelamento, timeout e erros de spawn/espera. stdin é nulo para FFmpeg/ffprobe; argumentos continuam separados, sem shell. |
| Limites | Timeout parametrizado nos testes e diagnóstico limitado a 64 KiB; nenhum processo abandonado deve permanecer após cancelamento. |
| Compatibilidade | `-nostdin` permanece somente no FFmpeg; ffprobe evita a opção incompatível e usa `Stdio::null()`. |
| Regressão UI | Smoke JPEG XL corrigido para a posição atual do menu contextual; conversão real voltou a criar saída JXL e passou por ffprobe. |
| Qualidade | Testes fake de cancelamento/timeout/argumentos, stress de 20 rodadas e validação cross-Windows adicionados. |

## [0.1.11] — 2026-08-19

A versão 0.1.11 consolida os lotes P1 de exploração e entrega a primeira distribuição portable verificável para Windows x86-64.

| Área | Mudança |
|---|---|
| Preview | Preview seguro de imagens e texto literal com limites de memória/bytes, BOM UTF-8/UTF-16, rejeição de binários/symlinks e cache cancelável. |
| Busca | Busca recursiva em worker com batches, limites, cancelamento e geração anti-stale. |
| Operações | Drag-and-drop, clipboard, propriedades e nova pasta preservados; exclusão Windows envia arquivos à Lixeira via Shell API sem fallback permanente silencioso. |
| Configurações | Preferências por usuário em schema v1, escrita atômica, fallback para defaults e smoke de restauração. |
| Distribuição | ZIP portable `rovex-v0.1.11-windows-x86_64-portable.zip`, manifesto, SHA-256 e verificador de conteúdo; artefato explicitamente não assinado. |
| Qualidade | Testes, Clippy estrito, audit/deny, cross-build Windows GNU, manifesto PE, smoke UI e empacotamento reproduzível passaram. |

## [0.1.10] — 2026-08-18

A versão 0.1.10 publica a primeira etapa do master prompt do issue #2: auditoria Foundation, plano incremental e documentação reconciliada, sem adicionar funcionalidades não auditadas.

| Área | Mudança |
|---|---|
| Auditoria | Criado `./docs/audits/ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md` com arquitetura, módulos, recursos, lacunas, riscos, testes, dependências e ordem recomendada. |
| Roadmap | Criado `./docs/plans/issue-2-execution-plan.md` com fases Foundation, Core Explorer, Search, Preview, Advanced Tools, Windows Integration e Distribution. |
| Documentação | README atualizado para refletir a release pública v0.1.9 e diferenciar o estado da tag dos refinamentos posteriores da branch principal. |
| Qualidade | Auditoria, `cargo fmt`, `cargo check` e suíte de testes executados antes da publicação. |

## [0.1.9] — 2026-08-17

A versão 0.1.9 conclui a modularização arquitetural do Rovex sem alterar os fluxos funcionais da v0.1.8.

| Área | Mudança |
|---|---|
| Arquitetura Rust | Operações, estado desktop, jobs, conversores e handlers foram divididos em módulos coesos, com fachadas pequenas e encapsulamento preservado. |
| Interface Slint | `main.slint` foi reduzido a 317 linhas; tokens, controles, modelos, toolbar e overlays agora vivem em módulos próprios. |
| Compatibilidade | Corrigidos imports do backend Windows após a migração para `converters::backend`; o check `x86_64-pc-windows-gnu` passou. |
| Build | `build.rs` agora observa todos os módulos Slint importados com `cargo:rerun-if-changed`. |
| Qualidade | Todos os arquivos de produção ficaram abaixo de 400 linhas; `cargo fmt`, check, testes, Clippy, auditorias, smoke tests e build release passaram. |

## [0.1.8] — 2026-08-17

A versão 0.1.8 refatora a interface visual sem remover recursos existentes e fecha o issue aberto de navegação com abas reais.

| Área | Mudança |
|---|---|
| Identidade | Novo ícone Rovex em PNG/ICO, incorporado ao executável Windows e acompanhado de desktop entry Linux. |
| Tema | Tokens escuros semânticos, superfícies diferenciadas, raios médios, espaçamento revisado e variantes de ação primária/perigosa. |
| Toolbar | Cabeçalho Rovex, navegação mais espaçada e Atualizar substituído por ícone de reload com tooltip e label acessível. |
| Lista | Marcadores visuais por tipo/extensão, labels acessíveis e preservação de nome, detalhes e seleção. |
| Abas | Abertura, seleção e fechamento de abas com histórico independente por aba; voltar/avançar continuam preservados. |
| Menu | Melhor hierarquia das ações, exclusão destacada como perigosa e conversões mantidas com estados corretos. |
| Regressões | Corrigida a identificação de arquivos regulares após a troca dos marcadores `[FILE]`/`[DIR]` por categorias semânticas. Scripts gráficos foram atualizados para a nova geometria. |
| Validação | 43 testes passaram, conversões reais passaram, smoke tests de abas e JPEG XL passaram, `cargo check`, Clippy, auditorias e cross-build Windows passaram. |

## [0.1.7] — 2026-08-17

A versão 0.1.7 corrige a descoberta quando o binário do Rovex e a imagem a converter estão em pastas diferentes e amplia a recuperação para instalações informadas por pasta ou variáveis `FFMPEG_*`.

| Área | Correção |
|---|---|
| Descoberta | O diretório de trabalho atual agora é candidato independente do diretório do executável. |
| Overrides | `ROVEX_FFMPEG_PATH`/`ROVEX_FFPROBE_PATH`, App Paths e variáveis `FFMPEG_HOME`, `FFMPEG_ROOT`, `FFMPEG_DIR` e `FFMPEG_PATH` aceitam arquivo direto, arquivo sem extensão ou pasta contendo o backend. |
| Regressão | Novo teste unitário cobre pastas de instalação sem extensão e novo teste gráfico copia o binário para uma pasta diferente da imagem. |
| Validação | Conversão real JPEG XL foi executada pela UI em Xvfb, com saída criada na pasta da imagem e codec confirmado como `jpegxl` pelo ffprobe. |

## [0.1.6] — 2026-08-17

A versão 0.1.6 organiza a documentação do estado atual, alinha o README com a descoberta em camadas do FFmpeg/ffprobe e repete a validação completa antes da distribuição.

| Área | Resultado |
|---|---|
| Organização | README, documentação da release e referências de versão alinhados com o estado publicado. |
| Conversão | Camadas de App Paths, `SearchPathW`, WinGet, Chocolatey, Scoop e `where.exe` preservadas e verificadas. |
| Qualidade | `cargo check`, testes, Clippy, auditorias e cross-build Windows repetidos sem erros bloqueantes. |
| Distribuição | Builds release Linux/Windows e checksums serão gerados para a v0.1.6. |

## [0.1.5] — 2026-08-17

A versão 0.1.5 amplia a recuperação automática dos backends de conversão no Windows e transforma a conversão em uma cascata real de candidatos verificáveis.

| Área | Correção |
|---|---|
| Registro | Leitura somente leitura de App Paths por usuário e por máquina, incluindo valores expandidos e visões de Registro 32/64 bits. |
| Busca nativa | Uso de `SearchPathW` com buffer redimensionável e validação de caminho absoluto. |
| Gerenciadores | Busca limitada em WinGet Links/Packages, Chocolatey, Scoop, shims e instalações lado a lado. |
| Recuperação | `where.exe` do System32 como fallback final, sem `/r`, sem shell command concatenado e sem download em runtime. |
| Execução | Tentativas reais por candidatos `ffmpeg`/`ffprobe`, limpeza de arquivos temporários e publicação somente após validação pelo codec. |
| Segurança | Diretórios e caminhos relativos são recusados como backends; symlinks/junctions só são aceitos quando resolvem para arquivos regulares. |
| Testes | Regressões para diretórios, caminhos relativos e links; conversões reais JPEG XL, PNG, Opus e FLAC validadas pelo ffprobe. |

## [0.1.4] — 2026-08-17

A versão 0.1.4 é uma nova reconstrução estável do estado corrigido da v0.1.3, repetindo a validação completa e publicando binários otimizados Linux/Windows.

| Área | Resultado |
|---|---|
| Compilação | `cargo check` e `cargo build --release` executados no estado atual sem erros. |
| Compatibilidade | Builds Linux x86_64 e Windows GNU x86_64 regenerados. |
| Qualidade | Testes, Clippy, auditorias e cross-check Windows repetidos antes da publicação. |
| Distribuição | Pacotes e SHA-256 regenerados e verificados após download da release. |

## [0.1.3] — 2026-08-17

A versão 0.1.3 corrige a falha observada em ambiente real na descoberta dos backends de conversão.

| Área | Correção |
|---|---|
| Descoberta | O PATH persistente do usuário e do sistema Windows agora é lido diretamente do Registro, incluindo valores `REG_EXPAND_SZ`, mesmo quando o processo foi iniciado antes da instalação do FFmpeg. |
| Compatibilidade | Symlinks, junctions e links de gerenciadores de pacotes que apontam para um arquivo regular agora são aceitos como executáveis válidos. |
| Recuperação | Depois de encontrar o FFmpeg, o Rovex tenta também o `ffprobe` no mesmo diretório antes de falhar. |
| Testes | O teste real de quatro formatos passa com PATH e overrides deliberadamente inválidos, usando os fallbacks seguros do sistema. |
| Qualidade | O ciclo completo de formatação, check, testes, Clippy, auditorias e cross-check Windows foi executado após a correção. |

## [0.1.2] — 2026-08-16

A versão 0.1.2 corrige a descoberta dos backends de conversão, melhora a ergonomia da toolbar, amplia os locais padrão do Windows e arredonda visualmente a sidebar.

| Área | Correção |
|---|---|
| Conversão | Resolução determinística e segura de `ffmpeg`/`ffprobe` por PATH, diretório do Rovex, locais fixos de instalação e overrides absolutos; ausência do backend agora informa o número de tentativas sem declarar sucesso falso. |
| Toolbar | Largura mínima e espaçamento explícito para voltar, avançar, subir e atualizar. |
| Locais | Known Folders oficiais do Windows para Área de Trabalho, Documentos, Downloads, Imagens, Vídeos, Músicas e Objetos 3D, com filtro por diretório existente e fallback portátil. |
| Interface | Painel de Locais e itens selecionados com bordas arredondadas e clip interno no tema escuro. |
| Dependências | `windows-sys` 0.61.2 restrito ao alvo Windows para `SHGetKnownFolderPath`. |
| Qualidade | `cargo check`, testes, Clippy, auditorias, builds Linux/Windows e CI remoto executados. |

## [0.1.1] — 2026-08-15

A versão 0.1.1 adiciona um fluxo completo de conversão multimídia integrado à interface nativa Slint, sem simulação e sem download de executáveis em runtime.

| Área | Mudança |
|---|---|
| Interface | Tema escuro consistente com tokens locais para superfícies, textos, bordas, seleção, inputs, botões e modais. |
| Menu contextual | Clique direito em uma linha seleciona o item e abre ações de arquivo e conversão. |
| Imagens | JPEG, PNG e formatos de imagem compatíveis podem ser convertidos para JPEG XL via FFmpeg/libjxl. |
| Áudio | WAV, MP3, FLAC e formatos de áudio compatíveis podem ser convertidos para Opus via FFmpeg/libopus. |
| Conversões adicionais | Imagens podem ser convertidas para PNG e áudio para FLAC quando a extensão de origem é compatível. |
| Segurança | Origem e destino são validados; a saída é criada no mesmo diretório, nunca sobrescreve arquivo existente e só é publicada após validação por tamanho e codec via ffprobe. |
| Concorrência | Conversões executam em worker dedicado, com progresso por fases, cancelamento cooperativo e atualização da pasta após conclusão. |
| Compatibilidade | O código foi verificado no alvo `x86_64-pc-windows-gnu`; as asserções de caminho dos testes são portáveis entre POSIX e Windows. |
| Qualidade | Formatação, compilação, testes, Clippy, cargo-audit, cargo-deny, build release, smoke test gráfico e teste end-to-end de conversão foram executados. |

As conversões dependem de `ffmpeg` e `ffprobe` disponíveis. O Rovex tenta o `PATH`, o diretório do executável, o diretório de trabalho e locais seguros de instalações comuns; `ROVEX_FFMPEG_PATH` e `ROVEX_FFPROBE_PATH` podem apontar para executáveis absolutos em diagnóstico controlado. No ambiente de desenvolvimento, o backend validado foi FFmpeg/ffprobe 6.1.1 do Ubuntu 24.04, com libjxl, libopus, PNG e FLAC disponíveis.

## [0.1.0] — 2026-08-15

Primeira release portable com navegação local, listagem real, seleção, filtro local, histórico, operações seguras de copiar/mover/renomear/excluir, workers limitados, cancelamento cooperativo, compatibilidade inicial com Windows 10/11 e interface Slint 1.17.1.
