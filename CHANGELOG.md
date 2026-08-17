# Changelog

Todas as mudanças relevantes do Rovex são registradas neste arquivo.

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
