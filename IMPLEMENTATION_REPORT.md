# Relatório de implementação e hardening — Rovex

## Resultado atual

O Rovex evoluiu da fundação Rust para um protótipo desktop funcional. O binário agora abre uma janela Slint, recebe um diretório inicial, lista entradas reais, navega por caminho, sobe para a pasta pai, atualiza a listagem e mostra status ou erro controlado. A lista é atualizada no thread principal do Slint; o filesystem roda em workers nomeados.

A interface não é apresentada como um Explorer completo. Pesquisa global, abas, thumbnails, pré-visualização, drag and drop, integração com shell, operações de arquivo disparadas pela UI, conversores, OCR, instalador, assinatura e atualização permanecem fora da primeira fatia e estão documentados como pendências. Histórico voltar/avançar, seleção múltipla local e uma barra lateral de locais existentes já são funcionais.

## Checkpoints e backups

| Branch | Ponto protegido |
|---|---|
| [`backup/pre-hardening-2026-08-14`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/pre-hardening-2026-08-14) | Estado anterior ao ciclo de hardening |
| [`backup/stable-d431ba7`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/stable-d431ba7) | Commit estável anterior ao hardening |
| [`backup/before-desktop-ui-2026-08-14`](https://github.com/danilo-jesus-unifil/Rovex/tree/backup/before-desktop-ui-2026-08-14) | Estado estável imediatamente anterior à integração Slint |
| `backup/before-history-selection-2026-08-14` | Checkpoint publicado antes de histórico e seleção múltipla |
| `backup/before-sidebar-2026-08-14` | Checkpoint publicado antes da barra lateral de locais |
| `backup/before-performance-refactor-2026-08-15` | Checkpoint publicado antes da refatoração de performance |
| `backup/before-final-audit-fixes-2026-08-15` | Checkpoint publicado antes da auditoria final de segurança e UX |

Os checkpoints `backup/before-history-selection-2026-08-14`, `backup/before-sidebar-2026-08-14`, `backup/before-performance-refactor-2026-08-15` e `backup/before-final-audit-fixes-2026-08-15`, além dos branches históricos anteriores, foram publicados no GitHub antes das respectivas alterações arriscadas.

## Implementação e correções

O núcleo mantém listagem por APIs de filesystem, classificação sem seguir links automaticamente, normalização de destinos, validação contra raiz e sobrescrita, cópia atômica, criação, renomeação e exclusão limitada a arquivos, links e diretórios vazios. Os testes de segurança incluem caminhos equivalentes, `..`, componentes finais ambíguos, links simbólicos e diretórios não vazios. As melhorias do prompt Rustora adicionam filtro local por nome, sem pesquisa recursiva, histórico de navegação com pilhas independentes de voltar/avançar, seleção múltipla por clique, Ctrl-clique, Shift-clique e Ctrl+A e uma barra lateral que apenas apresenta diretórios conhecidos e existentes. A etapa de performance adiciona um worker único latest-only para carregamento de filesystem, encerramento cooperativo dos workers e um Design System Slint local com tokens de cor, espaçamento, raio e estados. A auditoria final adiciona publicação de cópia sem sobrescrita em corrida, preservação de `PathBuf` para nomes Unicode inválidos, recusa de caminhos relativos e componentes pai symlink, mensagens humanizadas, empty states e navegação de sidebar por teclado.

A UI Slint 1.17.1 usa backend Winit, renderer software, acessibilidade e recursos mínimos, evitando renderizadores não utilizados. A barra de endereço dispara navegação somente ao confirmar, e não a cada tecla. O estado vazio diferencia pasta sem entradas de filtro sem resultados e nunca mascara erro de filesystem. O filtro opera sobre a pasta atual, usa fila latest-only com um worker dedicado e não cria uma thread por tecla. As linhas carregadas ficam em snapshots `Arc<[LoadedRow]>`, liberando o mutex antes da filtragem. O carregamento de filesystem usa um único `LoadScheduler` latest-only; navegações rápidas substituem pedidos pendentes, resultados obsoletos continuam protegidos por geração e o worker encerra cooperativamente quando seu scheduler é descartado. O histórico atualiza os botões voltar/avançar somente no event loop e descarta resultados de carregamentos obsoletos por geração. A seleção mantém chaves estáveis, intervalo inclusivo de Shift e estado visual por linha em `VecModel<FileRow>`, apenas no thread principal, conforme a API do Slint. A barra lateral usa um `VecModel<LocationRow>` pequeno, sem enumeração de drives, cálculo de espaço ou favoritos persistentes.

Durante a revisão foram encontrados e corrigidos erros reais: sintaxe de módulo no Slint, inferência de tipos do `VecModel`, conversão de `Cow<str>` para `SharedString`, acesso incorreto ao modelo, warning do Clippy por alocação desnecessária, testes temporários frágeis, comparação Windows entre caminho estendido e caminho curto, risco de resultados obsoletos sobrescreverem navegação recente e criação de uma thread por navegação. O carregamento agora usa geração atômica, fila latest-only e um único worker; falhas do worker e do modelo viram status controlado em vez de panic.

## Dependências e auditoria

A versão do Rust está fixada em `rust-toolchain.toml` com Rust 1.97.1, rustfmt, Clippy e alvo Windows x64. O Slint é fixado em 1.17.1; seu MSRV 1.92 é compatível com o toolchain do projeto.

`cargo deny check` está aprovado em advisories, bans, licenças e fontes. A política permite somente os identificadores observados na árvore, incluindo `LicenseRef-Slint-Royalty-free-2.0` e `LicenseRef-Slint-Software-3.0`, referências declaradas pelo próprio Slint. `cargo audit` termina com código 0 e não encontrou vulnerabilidades, mas reporta quatro warnings de manutenção transitivos: `bincode` 2.0.1, `paste` 1.0.15, `rustybuzz` 0.20.1 e `ttf-parser` 0.25.1. A base RustSec não fornece atualização segura para essa cadeia durante esta verificação; os warnings permanecem visíveis, registrados em [`docs/slint-research.md`](docs/slint-research.md) e não são tratados como vulnerabilidades.

## Verificações locais

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | **28 testes aprovados, 0 falhas; 1 benchmark manual ignorado por padrão** |
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
| Smoke de barra lateral | Aprovado; `Início` carregou `/home/ubuntu` e locais não existentes foram omitidos |
| Benchmark de filtro | Aprovado; 100.000 linhas em 9,732455 ms no release antes da otimização |
| Auditoria visual | Aprovada; tokens globais, estados de seleção, empty states, foco de sidebar e layout mínimo coerentes |
| Auditoria final de segurança/UX | Aprovada; race de cópia, paths Unicode/symlink, microcopy e estados de erro cobertos |

A listagem CLI foi validada com 100.000 arquivos sem crash; a UI foi validada com 10.000 arquivos e filtro local para um resultado. Os smoke tests release também confirmaram quatro linhas selecionadas, as transições voltar/avançar, a navegação pela barra lateral até `/home/ubuntu`, navegação por teclado até `/home/ubuntu/Downloads`, empty state em pasta vazia, a janela mínima 720×480 e os estados visuais do Design System. A linha de base do CLI foi `0,332518 s` e `30.356 KiB` de pico RSS; após o worker único, foi `0,348735 s` e `30.104 KiB`. O filtro manual sobre 100.000 linhas mediu `9,732455 ms` antes de qualquer mudança de normalização. A troca do loader é tratada como redução de concorrência e encerramento correto, não como ganho de tempo, pois a amostra isolada variou. O carregamento atual materializa metadados da pasta, enquanto a representação visual usa `ListView`; carregamento incremental de metadados permanece como melhoria futura para diretórios extremos.

O build Windows foi realizado com MinGW no ambiente Linux. Isso confirma compilação e formato do artefato, mas não substitui execução nativa em Windows 10/11, testes de DPI, acessibilidade, permissões Win32, junctions, UNC/SMB, instalador e desinstalador. A CI do commit `a9e32e8` concluiu com sucesso em Linux, Windows e auditoria de dependências na execução `31857990905`; o job Ubuntu instala `pkg-config` e `libfontconfig1-dev`, exigidos pelo backend de fontes do Slint.

## Auditoria manual

A busca por `unsafe`, `TODO`, `FIXME`, `panic!`, `unwrap` e `expect` encontrou `expect` somente em auxiliares de teste e em uma asserção de preparação de fixture. Não há `unsafe` nem caminhos de produção dependentes de `panic`. Os usos de `println!` e `eprintln!` ficam restritos ao modo CLI e ao diagnóstico de inicialização.

## Próximo gate

A CI remota da auditoria final passou em Linux, Windows e auditoria de dependências na execução `31857990905`. Antes de anunciar compatibilidade final, a execução manual em Windows deve cobrir DPI, teclado, acessibilidade, permissões, reparse points, paths longos e arquivos em uso. O próximo gate de produto é conectar operações reais de arquivo à seleção com confirmação, cancelamento, progresso e testes de segurança. A cópia de núcleo já protege publicação sem sobrescrita inclusive em corrida; a UI ainda não dispara essa operação. A etapa de performance não adicionou previews, thumbnails, polling, hash ou pesquisa global; todos continuam sob demanda. Favoritos persistentes e seleção de drives ficam para uma etapa posterior, sem análise automática no startup. Pesquisa global, thumbnails e análise de espaço continuam explicitamente sob demanda, nunca no startup.

## Referências técnicas

A documentação oficial do Slint descreve a versão 1.17 como um passo para desktop e confirma recursos como drag and drop, tooltips e acessibilidade [1]. A API `Weak::upgrade_in_event_loop` é a forma usada para devolver resultados de workers ao event loop [2], e `ModelRc` deve ser manipulado no thread principal [3]. A documentação do cargo-deny confirma a declaração de `LicenseRef-*` e o modo `unmaintained = "workspace"` [4] [5]. A Microsoft recomenda declarar DPI awareness no manifesto do processo [6].

[1]: https://slint.dev/blog/slint-1.17-released "Slint 1.17 Released"
[2]: https://docs.slint.dev/latest/docs/rust/slint/struct.Weak "Slint Rust API — Weak"
[3]: https://docs.slint.dev/latest/docs/rust/slint/struct.ModelRc "Slint Rust API — ModelRc"
[4]: https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html "cargo-deny — Licenses configuration"
[5]: https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html "cargo-deny — Advisories configuration"
[6]: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process "Microsoft Learn — Setting the default DPI awareness for a process"
