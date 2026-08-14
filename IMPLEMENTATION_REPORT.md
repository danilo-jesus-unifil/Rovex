# Relatório de implementação e hardening — Rovex

## Resultado atual

O Rovex evoluiu da fundação Rust para um protótipo desktop funcional. O binário agora abre uma janela Slint, recebe um diretório inicial, lista entradas reais, navega por caminho, sobe para a pasta pai, atualiza a listagem e mostra status ou erro controlado. A lista é atualizada no thread principal do Slint; o filesystem roda em workers nomeados.

A interface não é apresentada como um Explorer completo. Pesquisa global, abas, thumbnails, pré-visualização, drag and drop, integração com shell, operações de arquivo disparadas pela UI, conversores, OCR, instalador, assinatura e atualização permanecem fora da primeira fatia e estão documentados como pendências. Histórico voltar/avançar e seleção múltipla local já são funcionais.

## Checkpoints e backups

| Branch | Ponto protegido |
|---|---|
| [`backup/pre-hardening-2026-08-14`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/pre-hardening-2026-08-14) | Estado anterior ao ciclo de hardening |
| [`backup/stable-d431ba7`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/stable-d431ba7) | Commit estável anterior ao hardening |
| [`backup/before-desktop-ui-2026-08-14`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/before-desktop-ui-2026-08-14) | Estado estável imediatamente anterior à integração Slint |
| `backup/before-history-selection-2026-08-14` | Checkpoint local criado antes de histórico e seleção múltipla |

Os três branches históricos anteriores foram publicados no GitHub antes das respectivas alterações arriscadas. O checkpoint `backup/before-history-selection-2026-08-14` foi criado localmente antes desta fatia e será publicado junto com o incremento.

## Implementação e correções

O núcleo mantém listagem por APIs de filesystem, classificação sem seguir links automaticamente, normalização de destinos, validação contra raiz e sobrescrita, cópia atômica, criação, renomeação e exclusão limitada a arquivos, links e diretórios vazios. Os testes de segurança incluem caminhos equivalentes, `..`, componentes finais ambíguos, links simbólicos e diretórios não vazios. As melhorias do prompt Rustora adicionam filtro local por nome, sem pesquisa recursiva, histórico de navegação com pilhas independentes de voltar/avançar e seleção múltipla por clique, Ctrl-clique, Shift-clique e Ctrl+A.

A UI Slint 1.17.1 usa backend Winit, renderer software, acessibilidade e recursos mínimos, evitando renderizadores não utilizados. A barra de endereço dispara navegação somente ao confirmar, e não a cada tecla. O filtro opera sobre a pasta atual, usa fila latest-only com um worker dedicado e não cria uma thread por tecla. As linhas carregadas ficam em snapshots `Arc<[LoadedRow]>`, liberando o mutex antes da filtragem. O histórico atualiza os botões voltar/avançar somente no event loop e descarta resultados de carregamentos obsoletos por geração. A seleção mantém chaves estáveis, intervalo inclusivo de Shift e estado visual por linha em `VecModel<FileRow>`, apenas no thread principal, conforme a API do Slint.

Durante a revisão foram encontrados e corrigidos erros reais: sintaxe de módulo no Slint, inferência de tipos do `VecModel`, conversão de `Cow<str>` para `SharedString`, acesso incorreto ao modelo, warning do Clippy por alocação desnecessária, testes temporários frágeis, comparação Windows entre caminho estendido e caminho curto e risco de resultados obsoletos sobrescreverem navegação recente. O carregamento agora usa geração atômica e descarta resultados antigos; falhas do worker e do modelo viram status controlado em vez de panic.

## Dependências e auditoria

A versão do Rust está fixada em `rust-toolchain.toml` com Rust 1.97.1, rustfmt, Clippy e alvo Windows x64. O Slint é fixado em 1.17.1; seu MSRV 1.92 é compatível com o toolchain do projeto.

`cargo deny check` está aprovado em advisories, bans, licenças e fontes. A política permite somente os identificadores observados na árvore, incluindo `LicenseRef-Slint-Royalty-free-2.0` e `LicenseRef-Slint-Software-3.0`, referências declaradas pelo próprio Slint. `cargo audit` termina com código 0 e não encontrou vulnerabilidades, mas reporta quatro warnings de manutenção transitivos: `bincode` 2.0.1, `paste` 1.0.15, `rustybuzz` 0.20.1 e `ttf-parser` 0.25.1. A base RustSec não fornece atualização segura para essa cadeia durante esta verificação; os warnings permanecem visíveis, registrados em [`docs/slint-research.md`](docs/slint-research.md) e não são tratados como vulnerabilidades.

## Verificações locais

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | **20 testes aprovados, 0 falhas** |
| `cargo clippy --all-targets --all-features -- -D warnings` | Aprovado |
| `cargo build --release` | Aprovado |
| `cargo build --release --target x86_64-pc-windows-gnu` | Aprovado |
| Artefato Windows | Validado como PE32+ x86-64 |
| `cargo deny check` | Aprovado; advisories, bans, licenças e fontes OK |
| `cargo audit` | Código 0; quatro warnings transitivos de manutenção documentados |
| Modo CLI `cargo run -- --cli .` | Aprovado; listagem real |
| Smoke UI em Xvfb | Aprovado; janela, `/tmp`, filtro `cargo`, cinco resultados, status e screenshot 1100×720 |
| Estresse CLI | Aprovado com 10.000, 50.000 e 100.000 arquivos temporários |
| Estresse UI | Aprovado com 10.000 arquivos e filtro para um resultado |
| Smoke de seleção | Aprovado; clique, Ctrl-clique, Shift-clique e Ctrl+A selecionaram quatro arquivos reais |
| Smoke de histórico | Aprovado; navegação para subpasta, voltar e avançar com caminhos reais |

A listagem CLI foi validada com 100.000 arquivos sem crash; a UI foi validada com 10.000 arquivos e filtro local para um resultado. Os smoke tests release também confirmaram quatro linhas selecionadas e as transições voltar/avançar. O carregamento atual materializa metadados da pasta, enquanto a representação visual usa `ListView`; carregamento incremental de metadados permanece como melhoria futura para diretórios extremos.

O build Windows foi realizado com MinGW no ambiente Linux. Isso confirma compilação e formato do artefato, mas não substitui execução nativa em Windows 10/11, testes de DPI, acessibilidade, permissões Win32, junctions, UNC/SMB, instalador e desinstalador. A CI do commit `613b41d` concluiu com sucesso em Linux, Windows e auditoria de dependências na execução `31851551574`; o job Ubuntu instala `pkg-config` e `libfontconfig1-dev`, exigidos pelo backend de fontes do Slint.

## Auditoria manual

A busca por `unsafe`, `TODO`, `FIXME`, `panic!`, `unwrap` e `expect` encontrou `expect` somente em auxiliares de teste e em uma asserção de preparação de fixture. Não há `unsafe` nem caminhos de produção dependentes de `panic`. Os usos de `println!` e `eprintln!` ficam restritos ao modo CLI e ao diagnóstico de inicialização.

## Próximo gate

A CI remota do incremento anterior passou em Linux, Windows e auditoria de dependências. Antes de anunciar compatibilidade final, a execução manual em Windows deve cobrir DPI, teclado, acessibilidade, permissões, reparse points, paths longos e arquivos em uso. O próximo gate de produto é conectar operações reais de arquivo à seleção com confirmação, cancelamento, progresso e testes de segurança. Pesquisa global, thumbnails e análise de espaço continuam explicitamente sob demanda, nunca no startup.

## Referências técnicas

A documentação oficial do Slint descreve a versão 1.17 como um passo para desktop e confirma recursos como drag and drop, tooltips e acessibilidade [1]. A API `Weak::upgrade_in_event_loop` é a forma usada para devolver resultados de workers ao event loop [2], e `ModelRc` deve ser manipulado no thread principal [3]. A documentação do cargo-deny confirma a declaração de `LicenseRef-*` e o modo `unmaintained = "workspace"` [4] [5]. A Microsoft recomenda declarar DPI awareness no manifesto do processo [6].

[1]: https://slint.dev/blog/slint-1.17-released "Slint 1.17 Released"
[2]: https://docs.slint.dev/latest/docs/rust/slint/struct.Weak "Slint Rust API — Weak"
[3]: https://docs.slint.dev/latest/docs/rust/slint/struct.ModelRc "Slint Rust API — ModelRc"
[4]: https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html "cargo-deny — Licenses configuration"
[5]: https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html "cargo-deny — Advisories configuration"
[6]: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process "Microsoft Learn — Setting the default DPI awareness for a process"
