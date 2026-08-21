# Rovex: Release Notes de Modernização

## 15 de agosto de 2026: registro histórico da modernização

O projeto migrou da edição Rust 2021 para a edição Rust 2024, mantendo Rust 1.97.1, que já era a release estável verificada. Slint e Slint-build continuam em 1.17.1, alinhados entre runtime e compiler.

O lockfile recebeu somente duas atualizações transitivas compatíveis: `uuid` 1.24.0 para 1.24.1 e `wayland-backend` 0.3.16 para 0.3.17. Nenhuma dependência direta redundante foi encontrada ou adicionada. As features do Slint continuam mínimas por plataforma: renderer de software, acessibilidade, compatibilidade 1.2 e backend Winit apropriado ao alvo; no Linux de CI, X11-only evita Wayland não exercitado.

O CI agora executa `cargo check` explicitamente e possui um job dedicado de cross-build Windows GNU, além dos jobs Linux, Windows e auditoria de supply chain. O código foi ajustado para os lints idiomáticos do Rust moderno sem silenciar warnings.

A auditoria não encontrou vulnerabilidades exploráveis. Quatro avisos de manutenção transitivos da cadeia Slint continuam visíveis e documentados: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. Eles não foram convertidos em exceções silenciosas.

Na data deste registro, a modernização passou com 31 testes aprovados, Clippy estrito, builds release Linux/Windows GNU, `cargo audit`, `cargo deny check` e smoke tests da interface e das operações de arquivo. A suíte evoluiu posteriormente para 45 testes pelo harness, com 43 aprovados e 2 ignorados explicitamente; o estado atual está em [`../reference/testing.md`](../reference/testing.md). A execução nativa em Windows 10 22H2, testes de DPI, manifesto PE, UNC/SMB, reparse points e paths longos permanecem como gates posteriores.
