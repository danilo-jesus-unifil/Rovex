# Estratégia de testes

## Ciclo local atual

O núcleo e a primeira camada desktop são verificados com `cargo fmt --all -- --check`, `cargo check --all-targets --all-features`, `cargo test --all-targets --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release`, build release cruzado para Windows x64, `cargo audit` e `cargo deny check`.

O resultado atual é de **16 testes aprovados, zero falhas**, Clippy sem diagnósticos e cargo-deny aprovado em advisories, bans, licenças e fontes. O cargo-audit termina com código 0 e informa quatro warnings de manutenção transitivos do Slint: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. Esses avisos permanecem visíveis e não são tratados como vulnerabilidades.

## Cobertura do núcleo

Os testes cobrem classificação de diretórios e arquivos, listagem sem seguir links simbólicos, recusa de listagem sobre arquivo, recusa de destino existente por padrão, recusa de operação na raiz, normalização de `..`, detecção de origem e destino equivalentes, rejeição de componente final ambíguo, cópia com arquivo temporário e validação do tamanho, renomeação, criação de diretório, exclusão de arquivo e recusa de exclusão de diretório não vazio.

## Cobertura desktop

A camada desktop tem testes para formatação de tamanhos, descoberta segura da pasta pai, carregamento de diretório real e conversão de erro de filesystem em status controlado. O worker utiliza uma geração atômica para descartar resultados de carregamentos obsoletos, e a atualização do `VecModel` ocorre apenas pelo event loop do Slint.

Um smoke test executável em Xvfb abre a janela release, localiza o título `Rovex`, edita a barra de endereço para `/tmp`, confirma a navegação, captura uma imagem de 1100×720 e encerra o processo. O screenshot validado mostra o caminho `/tmp`, a listagem real e o status de itens carregados.

## Próximos testes do filesystem

A próxima expansão deve incluir nomes Unicode e reservados do Windows, paths longos, arquivos maiores que 4 GB quando o runner permitir, junctions e demais reparse points, arquivos em uso, permissões negadas, unidades desconectadas, caminhos UNC, colisões de nome, interrupção de cópia, disco cheio e cancelamento. Cada bug encontrado deverá gerar um teste de regressão.

## Testes da interface e da distribuição

A camada desktop precisará ser verificada em Windows 10 e 11 com teclado, foco, leitor de tela quando disponível, alto contraste, escalas de 100%, 125%, 150% e 200%, dois monitores com DPI diferente, tema claro/escuro, navegação de histórico, ativação de pastas, cancelamento e mensagens de erro. A distribuição exigirá instalação limpa, upgrade, reinstalação, versão portátil, assinatura, atualização verificada e desinstalação sem apagar documentos do usuário.

## Auditoria de dependências

A CI executa `cargo audit` e `cargo deny check`. A política está em [`deny.toml`](../deny.toml), o toolchain fixado está em [`rust-toolchain.toml`](../rust-toolchain.toml) e a decisão sobre recursos do Slint e avisos transitivos está em [`slint-research.md`](slint-research.md).
