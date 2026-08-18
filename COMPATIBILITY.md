# Rovex — Compatibility Matrix

**Status:** auditoria de modernização em 15 de agosto de 2026. Este documento distingue explicitamente o que é alvo de produto, o que foi compilado e o que foi executado em cada plataforma. Um build bem-sucedido não é tratado como prova de compatibilidade completa.

## Política de suporte

O produto é direcionado a **Windows 10 e Windows 11 x64**. Como referência operacional para o Windows 10, o projeto adota Windows 10 versão 22H2, OS build 19045, cuja identificação é registrada pela Microsoft [4]. Essa escolha define o baseline de teste; ela não transforma a execução em Linux ou um build cross em teste nativo de Windows.

A documentação oficial do Slint 1.17.1 lista Windows 10 x86-64 e Windows 11 x86-64/aarch64 entre as plataformas desktop testadas pelo toolkit [2]. O Rovex utiliza somente a arquitetura x86-64 no escopo atual.

| Ambiente | Política | Evidência disponível | Estado |
|---|---|---|---|
| Linux x86-64 | Plataforma de desenvolvimento e validação | Release nativo, testes, Clippy, Xvfb e smoke UI | **Verificado localmente** |
| Windows x86-64 GNU | Artefato cross para diagnóstico e CI | `cargo build --release --target x86_64-pc-windows-gnu` | **Verificado localmente; execução pendente** |
| Windows 10 22H2 x64 | Mínimo operacional escolhido | Nenhuma execução nativa nesta sessão | **Não verificado nativamente** |
| Windows 11 x64 | Plataforma de produto | Job Windows do CI compila/testa em runner hospedado, sem identificação do build exato no repositório | **CI verificado; matriz de versão pendente** |
| Windows ARM64 | Fora do escopo atual | Não há target nem empacotamento | **Não suportado** |

## Toolchain e integração de plataforma

O projeto fixa Rust 1.97.1 em `rust-toolchain.toml`, com `rustfmt`, `clippy` e o target cross GNU. A release oficial Rust 1.97.1 foi publicada em 16 de julho de 2026 e corrigiu uma possível miscompilação em otimização LLVM [1]. O `Cargo.toml` usa a edição Rust 2024 e declara `rust-version = "1.97"`; portanto, Rust 1.97 é a MSRV efetiva do Rovex nesta versão, independentemente de versões mínimas de dependências transitivas.

A UI usa Slint 1.17.1 com renderer de software, acessibilidade, `compat-1-2` e backend Winit. No Linux testado, somente `backend-winit-x11` é habilitado para o CI/Xvfb; no Windows, o backend Winit apropriado ao target permanece habilitado. O renderer de software é o fallback deliberado para máquinas sem GPU dedicada e não depende de uma API exclusiva do Windows 11.

| Camada | Decisão | Fallback ou limitação |
|---|---|---|
| Janela | Slint 1.17.1 + Winit | Nenhum backend WebView ou Electron |
| Renderização | `renderer-software` | Compatível com máquinas sem GPU dedicada; desempenho gráfico avançado não é prometido |
| Linux CI | `backend-winit-x11` | Wayland não é compilado nesse alvo; isso não define suporte de produto Linux |
| Windows | `backend-winit` no target Windows | APIs específicas de versão não são chamadas diretamente pelo núcleo Rovex |
| Filesystem | `std::fs` e `PathBuf` | UNC, reparse points, long paths e permissões Windows ainda exigem execução nativa |
| Privilégios | Nenhum pedido de administrador no código atual | Manifesto embutido no PE cross-compiled com `asInvoker`, DPI e long-path awareness; execução nativa ainda pendente |

## Manifesto, DPI e temas

O manifesto `assets/rovex.manifest` agora é incorporado pelo `build.rs` via `winres`. O executável Windows GNU release foi inspecionado e contém a seção `.rsrc` com `asInvoker`, DPI awareness (`PerMonitorV2`, `PerMonitor`, `System`) e `longPathAware`; `scripts/verify_windows_manifest.sh` reproduz essa verificação. Isso prova a incorporação no artefato cross-compiled, mas não substitui a execução nativa em Windows 10/11 nem a validação visual por escala e múltiplos monitores.

Não há chamada Win32 direta nem API exclusiva do Windows 11 no núcleo atual. A interface usa tokens próprios do Design System e não depende do tema do sistema para determinar cores essenciais. Os atalhos P1 de teclado — F2, Delete, Backspace, Alt+Left/Right, Ctrl+F, Ctrl+L, setas, Ctrl+A e Enter — estão implementados na listagem e foram exercitados sob Xvfb; leitor de tela, alto contraste, escalas de 100%, 125%, 150%, 175% e 200% e múltiplos monitores continuam pendentes de validação nativa em Windows 10 e Windows 11.

O P1 de arquivos ocultos agora usa `symlink_metadata`: no host, nomes iniciados por ponto são ocultos; no Windows, os atributos `HIDDEN` e `SYSTEM` também são lidos sem seguir links ou reparse points. O botão Ocultos alterna a visibilidade e identifica entradas como `Arquivo oculto`, `Pasta oculta` ou `Item do sistema`; nenhuma operação protegida é disparada automaticamente. O fluxo foi testado sob Xvfb, mas a confirmação de atributos NTFS reais e a validação nativa Windows 10/11 continuam pendentes.

O P1 de clipboard usa `copypasta` 0.10.2 como adapter real do clipboard do sistema e mantém o provider vivo no `AppContext`, requisito importante no X11 para que Cut/Paste continue disponível depois do callback. Ctrl+C, Ctrl+X e Ctrl+V serializam um payload tipado do Rovex e Paste reutiliza o scheduler de operações, com cancelamento, refresh e mensagens estruturadas. A interoperabilidade com Explorer e aplicações externas — incluindo formatos nativos como `CF_HDROP` e atalhos de link — ainda não é declarada como concluída.

O P1 de Propriedades abre um diálogo modal somente leitura pelo menu contextual. O handler exige exatamente um item selecionado e usa o snapshot `LoadedRow` já publicado, com nome, tipo, localização, tamanho, timestamps, atributos e estado de diretório; não segue links/reparse points, não relê o alvo e não altera ACLs ou conteúdo. O painel General/Security/Details usa `Flickable` e foi exercitado com nome Unicode e rolagem em `scripts/test_properties.sh`; a captura confirma que o processo permanece ativo e o arquivo não sofre mutação. ACLs detalhadas, atributos NTFS completos, links reais, escala de acessibilidade e execução nativa Windows 10/11 permanecem pendentes.

## Busca recursiva

O lote Search separa o filtro local da ação explícita `Buscar`. O engine faz traversal iterativo em worker dedicado, compara somente nomes, publica resultados em batches, usa gerações para descartar eventos obsoletos e distingue conclusão, cancelamento e limites de resultados/diretórios/entradas. `symlink_metadata` impede descer em symlinks; reparse points são bloqueados no Windows, e falhas parciais são contabilizadas sem derrubar a UI. O botão `Cancelar` e refresh/navegação cancelam o job ativo. Testes unitários cobrem Unicode/case-insensitive, ordem determinística, cancelamento, limites, ocultos, raiz relativa e symlink; `scripts/test_recursive_search.sh` confirmou dois resultados reais em níveis diferentes sem travamento. A busca por índice do Windows, desempenho nativo em árvores muito grandes e execução Windows 10/11 permanecem pendentes.

## Preview e thumbnails

O lote de preview decodifica somente imagens estáticas suportadas diretamente pelo crate `image` 0.25.10 — BMP, GIF, JPEG, PNG e WebP — com formato inferido pelo conteúdo, não apenas pela extensão. O núcleo rejeita symlink/reparse point, exige arquivo regular, limita entrada a 128 MiB, dimensões a 8192×8192 e alocação de decode a 64 MiB, depois reduz a imagem para no máximo 256px por lado. O worker dedicado coalesce requests, cancela gerações antigas e mantém cache LRU limitado a 128 entradas/32 MiB; falhas, formatos falsos e conteúdo corrompido mostram fallback textual/ícone genérico sem derrubar a UI. A seleção de um único arquivo abre painel escuro contido; seleção múltipla, pastas e navegação escondem/cancelam a prévia. `scripts/test_preview.sh` confirmou PNG válido, `.jpg` inválido, fechamento do painel e processo ativo; testes unitários cobrem limites e symlink. PDF, vídeo, áudio, Office, codecs externos, handlers COM/Explorer e preview nativo Windows permanecem deliberadamente fora deste lote e exigem workers/processos isolados posteriores.

## Drag-and-drop

O P1 de drag-and-drop usa os blocos `DropArea`/`DataTransfer` do Slint 1.17.1 para payloads internos e o filtro público `slint::winit_030::WinitWindowAccessor` para os eventos nativos `HoveredFile`/`DroppedFile` do Winit. O hover aceita somente arquivo absoluto existente e regular, mostra feedback escuro na listagem e o drop despacha uma operação `Copy` real pelo `OperationScheduler`, com progresso, cancelamento, publicação sem sobrescrita e refresh. O filtro passa `PathBuf` diretamente ao worker para não perder nomes Windows que não sejam UTF-8; a rota DataTransfer textual permanece validada antes de qualquer operação. `cargo check`, testes unitários do handler, Clippy, cross-build Windows GNU, manifesto e todos os smoke tests passaram. O arraste real de Explorer para a janela ainda requer execução nativa em Windows 10/11; Xvfb não consegue produzir uma fonte externa equivalente.

## Filesystem e Unicode

A camada de segurança recusa caminhos relativos, raízes e componentes pai ambíguos ou simbólicos nas operações sensíveis. Os testes locais agora cobrem preservação de nomes Unicode com espaços e pontuação, caminhos aninhados com mais de 260 bytes no host, pontos finais em sistemas que os suportam e nomes reservados via teste condicionado ao Windows. Isso não substitui a execução em NTFS, exFAT, FAT32, USB, UNC/SMB, junctions, reparse points, arquivos em uso, permissões negadas ou long paths nativos no Windows.

A movimentação usa rename no mesmo volume e fallback real de copiar-e-remover entre volumes. A cópia opera em blocos, publica sem sobrescrita e cancela cooperativamente. A UI mostra erro parcial em vez de declarar sucesso agregado quando um item falha.

## Gaps de compatibilidade

A próxima validação de plataforma deve executar o binário em Windows 10 22H2 e Windows 11 x64, identificar o build do sistema, testar DPI por monitor, teclado, leitor de tela, alto contraste, tema claro/escuro, caminhos Unicode e longos, UNC/SMB, volumes removíveis, reparse points e arquivos em uso. O PE cross-compiled já tem uma verificação automatizada do manifesto; a matriz nativa ainda deve confirmar execução sem privilégios administrativos, comportamento visual e comportamento de instalação.

> **Conclusão:** o Rovex possui uma stack compilável e uma política de fallback coerente para Windows 10/11, mas a conformidade nativa do sistema operacional permanece parcialmente não verificada. Isso é uma limitação registrada, não uma afirmação implícita de suporte já provado.

## Referências

[1]: https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/ "Announcing Rust 1.97.1 — Rust Blog"
[2]: https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/ "Desktop — Slint Documentation"
[3]: https://slint.dev/blog/slint-1.17-released "Slint 1.17 Released — Slint Blog"
[4]: https://learn.microsoft.com/en-us/windows/release-health/release-information "Windows release information — Microsoft Learn"
