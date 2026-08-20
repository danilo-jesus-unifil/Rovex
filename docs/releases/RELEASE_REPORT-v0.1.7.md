# Rovex v0.1.7 — Correção de conversão fora da pasta do binário

## Problema reproduzido

A falha apresentada pela interface informava que `ffmpeg` não havia sido encontrado depois de 134 localizações. O teste foi reproduzido com o binário gráfico e a imagem em diretórios distintos: o executável foi copiado para `/tmp/rovex-jxl-separate-dirs/bin/rovex`, enquanto a imagem foi criada em `/tmp/rovex-jxl-separate-dirs/images/entrada.png`. O teste não usou o diretório do executável como diretório da imagem.

O fluxo foi executado pela interface Slint em Xvfb, com menu contextual e confirmação de conversão. A saída esperada foi criada em `/tmp/rovex-jxl-separate-dirs/images/entrada.jxl` e validada pelo `ffprobe` como codec `jpegxl`.

## Correção aplicada

A investigação identificou uma lacuna de recuperação quando a instalação era informada como pasta, quando o caminho não tinha extensão ou quando o processo gráfico iniciava com diretório de trabalho diferente do diretório que continha o backend. O resolvedor agora trata cada override, valor App Paths e variável `FFMPEG_*` como arquivo, arquivo sem extensão e possível raiz de instalação.

Além das camadas já existentes, foram adicionados o diretório de trabalho atual e as variáveis absolutas `FFMPEG_HOME`, `FFMPEG_ROOT`, `FFMPEG_DIR` e `FFMPEG_PATH`. Uma raiz como `C:\ffmpeg` passa a gerar candidatos como `C:\ffmpeg\ffmpeg.exe` e `C:\ffmpeg\ffprobe.exe`, sem aceitar a pasta como executável. O diretório do binário, o diretório da imagem e os diretórios irmãos continuam sendo tratados separadamente.

A execução continua usando `Command::arg`/`args` separados, caminhos absolutos e validação de arquivo regular. Nenhum shell command é montado e nenhum executável é baixado em runtime.

## Testes de regressão

| Verificação | Resultado |
|---|---|
| Teste gráfico com binário e imagem em pastas diferentes | Passou: JXL criado na pasta da imagem. |
| Validação do resultado | Passou: `ffprobe` detectou `jpegxl`. |
| Teste unitário de override sem extensão/pasta | Passou. |
| `cargo fmt --all -- --check` | Passou. |
| `cargo check --all-targets --all-features` | Passou. |
| `cargo check --target x86_64-pc-windows-gnu --all-targets --all-features` | Passou. |
| `cargo test --all-targets --all-features` | 41 testes passaram; 2 foram ignorados por benchmark/conversão real explícita. |
| Teste real de quatro conversões | Passou: JPEG XL, PNG, Opus e FLAC. |
| Clippy Linux e Windows GNU com `-D warnings` | Passou. |
| `cargo audit` | Passou sem advisory de vulnerabilidade bloqueante; os quatro warnings transitivos de manutenção continuam documentados. |
| `cargo deny check` | Advisories, bans, licenças e sources passaram. |
| Build Linux release | Passou; ELF x86_64 stripped de aproximadamente 16 MiB. |
| Build Windows GNU release | Passou; PE32+ x86_64 stripped de aproximadamente 12 MiB. |

## Regressão permanente

O script [`scripts/test_ui_jxl_separate_dirs.sh`](../../scripts/test_ui_jxl_separate_dirs.sh) cria a imagem em uma pasta, copia o binário para outra, abre a interface gráfica, aciona JPEG XL e valida a saída. Ele pode ser executado com `./scripts/test_ui_jxl_separate_dirs.sh` em um ambiente Linux com Xvfb, xdotool, ImageMagick, FFmpeg e ffprobe.

A release v0.1.7 inclui a correção no commit de preparação correspondente e será publicada com binários Linux/Windows, checksums SHA-256 e este relatório.
