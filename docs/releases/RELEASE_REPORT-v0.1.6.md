# Rovex v0.1.6 — Relatório técnico de release

## Escopo

A v0.1.6 organiza a documentação do estado atual do Rovex e alinha o README com o resolvedor em camadas de `ffmpeg.exe` e `ffprobe.exe` implementado na v0.1.5. Nenhuma funcionalidade estável foi removida. A arrumação mantém a política de não baixar executáveis em runtime, não montar comandos por shell e não declarar conversões concluídas sem validação real.

## Arrumação aplicada

| Arquivo/área | Ajuste |
|---|---|
| `README.md` | Atualizado de v0.1.4 para v0.1.5 antes da nova versão, documentando overrides, PATH, PATH persistente, App Paths, `SearchPathW`, WinGet, Chocolatey, Scoop e `where.exe`. |
| `README.md` | Referência do relatório atualizada para `./RELEASE_REPORT-v0.1.5.md` e pesquisa de descoberta adicionada. |
| `README.md` | Descrição dos warnings do `cargo audit` corrigida para `bincode`, `paste`, `rustybuzz` e `ttf-parser`. |
| `CHANGELOG.md` | Nova entrada v0.1.6 com o escopo da organização e da validação repetida. |
| `Cargo.toml` / `Cargo.lock` | Versão sincronizada para 0.1.6. |

## Resolvedor de conversão preservado

O Rovex continua tentando, em ordem, overrides absolutos, PATH herdado, PATH persistente do Registro, App Paths por usuário e máquina, `SearchPathW`, diretório do executável, diretório irmão do backend, diretórios controlados de WinGet/Chocolatey/Scoop e `where.exe` como último recurso. Cada candidato é absoluto e precisa ser arquivo regular ou link/junction que resulte em arquivo regular. A conversão tenta pares reais de FFmpeg/ffprobe, remove temporários após falhas e publica somente depois da validação do arquivo e do codec.

## Validação local

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Passou. |
| `cargo check --all-targets --all-features` | Passou. |
| `cargo check --target x86_64-pc-windows-gnu --all-targets --all-features` | Passou. |
| `cargo test --all-targets --all-features` | 40 testes passaram; 2 foram ignorados por benchmark/conversão real explícita. |
| Teste real ignorado executado explicitamente | Passou; JPEG XL, PNG, Opus e FLAC foram gerados e validados por ffprobe. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passou. |
| Clippy no alvo Windows GNU | Passou. |
| `cargo audit` | Código de saída 0; sem advisory de vulnerabilidade bloqueante. Permanecem quatro warnings transitivos de manutenção permitidos pela configuração. |
| `cargo deny check` | Advisories, bans, licenças e sources aprovados. |
| `cargo build --release` | Passou; ELF Linux x86_64 stripped de aproximadamente 16 MiB. |
| `cargo build --release --target x86_64-pc-windows-gnu` | Passou; PE32+ Windows x86_64 stripped de aproximadamente 12 MiB. |
| Smoke test gráfico em Xvfb | Processo permaneceu ativo durante 8 segundos e foi encerrado pelo timeout controlado, sem saída de erro. |

## Dependências

O `cargo audit` atual identifica warnings transitivos de manutenção para `bincode`, `paste`, `rustybuzz` e `ttf-parser`, relacionados ao grafo do stack Slint. Não foi realizada atualização ampla sem evidência de compatibilidade, porque isso poderia alterar o renderer e a matriz Windows. O `cargo deny check` aprovou a política atual de advisories, bans, licenças e sources.

## Estado para publicação

A nova tag será `v0.1.6`. Os pacotes serão acompanhados de SHA-256 e deste relatório. A execução efetiva em Windows 10/11 real, incluindo DPI, permissões Win32, junctions, UNC/SMB e acessibilidade nativa, continua sendo uma validação posterior que exige um ambiente Windows interativo; o cross-build e a qualidade Windows GNU foram executados com sucesso.
