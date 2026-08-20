# Relatório de release — Rovex v0.1.2

**Data:** 2026-08-16  
**Repositório:** `danilo-jesus-unifil/Rovex`  
**Base:** commit de preparação da release após as correções de PATH, Known Folders e UI  
**Plataformas-alvo:** Windows 10/11 x64 e Linux usado para validação local

## Escopo

A v0.1.2 consolida as correções de robustez do conversor, a ergonomia da toolbar, os Known Folders do Windows e o acabamento visual da sidebar sobre a base da v0.1.1. A UI usa componentes locais `RovexButton` e `RovexLineEdit`, evitando dependência visual de controles nativos claros. O menu contextual é aberto por clique direito em uma linha e mantém as ações habilitadas conforme a natureza e a extensão da entrada.

O módulo `src/converters.rs` executa FFmpeg por `std::process::Command`, sem shell e sem interpolação de comandos. O conversor valida a origem, recusa links simbólicos como entrada de conversão, calcula o destino irmão com extensão de saída, recusa destinos já existentes, escreve em arquivo temporário do mesmo diretório, valida tamanho e codec com ffprobe e publica por operação sem sobrescrita. Os processos têm limite de cinco minutos e respondem ao cancelamento cooperativo.

## Conversores disponíveis

| Origem compatível | Saída | Backend e validação |
|---|---|---|
| JPEG, PNG e formatos de imagem reconhecidos | JPEG XL (`.jxl`) | FFmpeg com `libjxl`; ffprobe espera `jpegxl`. |
| WAV, MP3, FLAC e formatos de áudio reconhecidos | Opus (`.opus`) | FFmpeg com `libopus`; ffprobe espera `opus`. |
| Formatos de imagem reconhecidos | PNG (`.png`) | FFmpeg com codec PNG; ffprobe espera `png`. |
| Formatos de áudio reconhecidos | FLAC (`.flac`) | FFmpeg com codec FLAC; ffprobe espera `flac`. |

O backend foi testado com **FFmpeg/ffprobe 6.1.1 no Ubuntu 24.04**. Em Windows, o usuário precisa instalar uma distribuição confiável que forneça `ffmpeg.exe` e `ffprobe.exe`. O Rovex tenta o `PATH`, o diretório do executável e locais seguros de instalações comuns; os overrides absolutos `ROVEX_FFMPEG_PATH` e `ROVEX_FFPROBE_PATH` ficam disponíveis para diagnóstico. O Rovex não baixa nem atualiza executáveis em runtime.

## Validação executada

A verificação local passou por `cargo fmt --all -- --check`, `cargo check --all-targets --all-features`, `cargo test --all-targets --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo audit`, `cargo deny check`, `cargo build --release` e `cargo check --target x86_64-pc-windows-gnu --all-features`.

O teste externo de conversão criou PNG e WAV reais, executou JXL, PNG, Opus e FLAC, validou as saídas por ffprobe e confirmou que uma segunda tentativa não sobrescreve o destino existente. O teste gráfico em Xvfb confirmou inicialização, tema escuro, clique direito, habilitação condicional por extensão, confirmação modal e publicação JXL pela UI. O CI do GitHub Actions validou Linux, Windows, cross-build GNU, Clippy, testes e auditorias.

## Limitações explícitas

A release não inclui instalador, assinatura de código, atualização automática, pesquisa global, abas, thumbnails, drag and drop, conversão PDF ou OCR. A execução efetiva em hardware Windows 10/11, incluindo DPI, permissões Win32, junctions, caminhos UNC/SMB e acessibilidade nativa, permanece uma etapa de validação de distribuição. Nenhuma dessas capacidades é simulada pela v0.1.2.
