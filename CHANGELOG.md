# Changelog

Todas as mudanças relevantes do Rovex são registradas neste arquivo.

## [Unreleased]

Esta seção registra correções preparadas após a publicação da v0.1.1, ainda sem uma nova tag de release.

| Área | Correção |
|---|---|
| Conversão | Resolução determinística e segura de `ffmpeg`/`ffprobe` por PATH, diretório do Rovex, locais fixos de instalação e overrides absolutos; ausência do backend agora informa o número de tentativas sem declarar sucesso falso. |
| Toolbar | Largura mínima e espaçamento explícito para voltar, avançar, subir e atualizar. |
| Locais | Known Folders oficiais do Windows para Área de Trabalho, Documentos, Downloads, Imagens, Vídeos, Músicas e Objetos 3D, com filtro por diretório existente e fallback portátil. |
| Dependências | `windows-sys` 0.61.2 restrito ao alvo Windows para `SHGetKnownFolderPath`. |

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
