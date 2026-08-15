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

O projeto fixa Rust 1.97.1 em `rust-toolchain.toml`, com `rustfmt`, `clippy` e o target cross GNU. A release oficial Rust 1.97.1 foi publicada em 16 de julho de 2026 e corrigiu uma possível miscompilação em otimização LLVM [1]. O `Cargo.toml` usa a edição Rust 2024 e preserva `rust-version = "1.92"` como MSRV declarada do pacote, compatível com o MSRV publicado pelo Slint 1.17.1.

A UI usa Slint 1.17.1 com renderer de software, acessibilidade, `compat-1-2` e backend Winit. No Linux testado, somente `backend-winit-x11` é habilitado para o CI/Xvfb; no Windows, o backend Winit apropriado ao target permanece habilitado. O renderer de software é o fallback deliberado para máquinas sem GPU dedicada e não depende de uma API exclusiva do Windows 11.

| Camada | Decisão | Fallback ou limitação |
|---|---|---|
| Janela | Slint 1.17.1 + Winit | Nenhum backend WebView ou Electron |
| Renderização | `renderer-software` | Compatível com máquinas sem GPU dedicada; desempenho gráfico avançado não é prometido |
| Linux CI | `backend-winit-x11` | Wayland não é compilado nesse alvo; isso não define suporte de produto Linux |
| Windows | `backend-winit` no target Windows | APIs específicas de versão não são chamadas diretamente pelo núcleo Rovex |
| Filesystem | `std::fs` e `PathBuf` | UNC, reparse points, long paths e permissões Windows ainda exigem execução nativa |
| Privilégios | Nenhum pedido de administrador no código atual | Manifesto Windows ainda não está embutido; `asInvoker`, DPI e long-path awareness são gate de distribuição |

## Manifesto, DPI e temas

A auditoria encontrou `build.rs`, `Cargo.toml` e código Rust, mas **não encontrou um manifesto Windows embutido**. Portanto, o projeto não declara neste momento conformidade com `asInvoker`, DPI awareness ou long-path awareness. Esses itens não serão considerados resolvidos por documentação; precisam de um manifesto efetivamente incorporado ao PE e de testes nativos.

Não há chamada Win32 direta nem API exclusiva do Windows 11 no núcleo atual. A interface usa tokens próprios do Design System e não depende do tema do sistema para determinar cores essenciais. Tema claro/escuro, alto contraste, escalas de 100%, 125%, 150%, 175% e 200% e múltiplos monitores continuam pendentes de validação visual em Windows 10 e Windows 11.

## Filesystem e Unicode

A camada de segurança recusa caminhos relativos, raízes e componentes pai ambíguos ou simbólicos nas operações sensíveis. Os testes locais cobrem nomes Unicode e preservação de `PathBuf`, mas não substituem testes em NTFS, exFAT, FAT32, USB, UNC/SMB, junctions, reparse points, arquivos em uso, permissões negadas ou caminhos longos no Windows.

A movimentação usa rename no mesmo volume e fallback real de copiar-e-remover entre volumes. A cópia opera em blocos, publica sem sobrescrita e cancela cooperativamente. A UI mostra erro parcial em vez de declarar sucesso agregado quando um item falha.

## Gaps de compatibilidade

A próxima validação de plataforma deve executar o binário em Windows 10 22H2 e Windows 11 x64, identificar o build do sistema, testar DPI por monitor, teclado, leitor de tela, alto contraste, tema claro/escuro, caminhos Unicode e longos, UNC/SMB, volumes removíveis, reparse points e arquivos em uso. Também deve verificar o PE final, o manifesto efetivamente incorporado, execução sem privilégios administrativos e comportamento de instalação.

> **Conclusão:** o Rovex possui uma stack compilável e uma política de fallback coerente para Windows 10/11, mas a conformidade nativa do sistema operacional permanece parcialmente não verificada. Isso é uma limitação registrada, não uma afirmação implícita de suporte já provado.

## Referências

[1]: https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/ "Announcing Rust 1.97.1 — Rust Blog"
[2]: https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/ "Desktop — Slint Documentation"
[3]: https://slint.dev/blog/slint-1.17-released "Slint 1.17 Released — Slint Blog"
[4]: https://learn.microsoft.com/en-us/windows/release-health/release-information "Windows release information — Microsoft Learn"
