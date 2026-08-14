# Estratégia de testes

## Ciclo local atual

O núcleo atual foi verificado com `cargo fmt --all -- --check`, `cargo test --all-targets --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release`, `cargo audit` e `cargo deny check`. O resultado atual é de **12 testes aprovados, zero falhas**, Clippy sem diagnósticos, auditoria RustSec sem advisories aplicáveis e política cargo-deny aprovada.

O binário de desenvolvimento também foi executado com `cargo run -- .` e listou um diretório real. O build cruzado `cargo build --release --target x86_64-pc-windows-gnu` foi concluído e o artefato foi identificado como PE32+ x86-64 para Windows.

## Cobertura da primeira fatia

Os testes cobrem classificação de diretórios e arquivos, listagem sem seguir links simbólicos, recusa de listagem sobre arquivo, recusa de destino existente por padrão, recusa de operação na raiz, normalização de `..`, detecção de origem e destino equivalentes, rejeição de componente final ambíguo, cópia com arquivo temporário e validação do tamanho, renomeação, criação de diretório, exclusão de arquivo e recusa de exclusão de diretório não vazio.

## Próximos testes do filesystem

A próxima expansão deve incluir nomes Unicode e reservados do Windows, paths longos, arquivos maiores que 4 GB quando o runner permitir, junctions e demais reparse points, arquivos em uso, permissões negadas, colisões de nome, interrupção de cópia, disco cheio, unidades desconectadas e caminhos UNC. Cada bug encontrado deverá gerar um teste de regressão.

## Testes da interface e da distribuição

A camada desktop precisará ser verificada com teclado, foco, leitor de tela quando disponível, alto contraste, escalas de 100%, 125%, 150% e 200%, dois monitores com DPI diferente, tema claro/escuro, navegação de histórico, cancelamento e mensagens de erro. A distribuição exigirá instalação limpa, upgrade, reinstalação, versão portátil, assinatura, atualização verificada e desinstalação sem apagar documentos do usuário.

## Auditoria de dependências

A CI executa `cargo audit` e `cargo deny`. Localmente, `cargo audit` 0.22.2 concluiu sem advisories para as dependências atuais, e `cargo deny check` 0.20.2 concluiu com advisories, bans, licenças e fontes OK. A política está em [`deny.toml`](../deny.toml), e o toolchain fixado está em [`rust-toolchain.toml`](../rust-toolchain.toml).
