# Estratégia de testes

## Ciclo local atual

O núcleo e a primeira camada desktop são verificados com `cargo fmt --all -- --check`, `cargo check --all-targets --all-features`, `cargo test --all-targets --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release`, build release cruzado para Windows x64, `cargo audit` e `cargo deny check`.

O resultado atual deste incremento é de **21 testes aprovados, zero falhas**, com um benchmark manual ignorado por padrão, Clippy sem diagnósticos e cargo-deny aprovado em advisories, bans, licenças e fontes. O cargo-audit termina com código 0 e informa quatro warnings de manutenção transitivos do Slint: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. Esses avisos permanecem visíveis e não são tratados como vulnerabilidades.

## Cobertura do núcleo

Os testes cobrem classificação de diretórios e arquivos, listagem sem seguir links simbólicos, recusa de listagem sobre arquivo, recusa de destino existente por padrão, recusa de operação na raiz, normalização de `..`, detecção de origem e destino equivalentes, rejeição de componente final ambíguo, cópia com arquivo temporário e validação do tamanho, renomeação, criação de diretório, exclusão de arquivo e recusa de exclusão de diretório não vazio.

## Cobertura desktop

A camada desktop tem testes para formatação de tamanhos, descoberta segura da pasta pai, carregamento de diretório real, conversão de erro de filesystem em status controlado, filtragem case-insensitive por nome, estado vazio sem resultados, histórico voltar/avançar e seleção por clique, Ctrl-clique, Shift-clique e Ctrl+A. O carregamento usa um worker único latest-only, geração atômica para descartar resultados obsoletos e encerramento cooperativo. O filtro usa uma fila latest-only, um worker dedicado e snapshots `Arc<[LoadedRow]>`, para que a UI não mantenha o mutex durante a filtragem. A atualização do `VecModel` ocorre apenas pelo event loop do Slint.

Um smoke test executável em Xvfb abre a janela release, localiza o título `Rovex`, edita a barra de endereço para `/tmp`, preenche o filtro com `cargo`, confirma a redução para cinco itens, captura uma imagem de 1100×720 e encerra o processo. O screenshot validado mostra o caminho `/tmp`, o filtro preenchido e a listagem reduzida.

Um segundo smoke test cria quatro arquivos temporários e executa clique normal, Ctrl-clique, Shift-clique e Ctrl+A. O screenshot `/tmp/rovex-selection-smoke.png` mostra as quatro linhas selecionadas e o status `4 itens selecionados`. Um terceiro smoke test cria uma raiz e uma subpasta, navega para dentro e confirma voltar/avançar por screenshots com caminhos reais em `/tmp/rovex-history-back.png` e `/tmp/rovex-history-forward.png`. Um quarto smoke test inicia em `/tmp`, clica no local `Início` e confirma o carregamento real de `/home/ubuntu`, sem análise de unidades ou espaço.

## Estresse e próximos testes do filesystem

A listagem CLI foi exercitada com 10.000, 50.000 e 100.000 arquivos temporários, retornando todas as entradas sem crash. A UI foi exercitada com 10.000 arquivos, e o filtro local reduziu a lista para um resultado sem travar a janela. A linha de base CLI de 100.000 arquivos foi `0,332518 s` e `30.356 KiB` de pico RSS; após o worker único foi `0,348735 s` e `30.104 KiB`, sem declarar ganho de tempo a partir de uma única amostra. O benchmark manual do filtro processou 100.000 linhas em `9,732455 ms` no release antes de qualquer mudança de normalização. O `ListView` mantém a representação visual limitada aos itens visíveis, mas o carregamento atual ainda materializa os metadados da pasta em memória; virtualização incremental do carregamento continua sendo uma melhoria futura para diretórios extremos.

A próxima expansão deve incluir nomes Unicode e reservados do Windows, paths longos, arquivos maiores que 4 GB quando o runner permitir, junctions e demais reparse points, arquivos em uso, permissões negadas, unidades desconectadas, caminhos UNC, colisões de nome, interrupção de cópia, disco cheio e cancelamento. Cada bug encontrado deverá gerar um teste de regressão.

## Testes da interface e da distribuição

A camada desktop precisará ser verificada em Windows 10 e 11 com teclado, foco, leitor de tela quando disponível, alto contraste, escalas de 100%, 125%, 150% e 200%, dois monitores com DPI diferente, tema claro/escuro, navegação de histórico, ativação de pastas, cancelamento e mensagens de erro. A distribuição exigirá instalação limpa, upgrade, reinstalação, versão portátil, assinatura, atualização verificada e desinstalação sem apagar documentos do usuário.

## Auditoria de dependências

A CI executa `cargo audit` e `cargo deny check`. A política está em [`deny.toml`](../deny.toml), o toolchain fixado está em [`rust-toolchain.toml`](../rust-toolchain.toml) e a decisão sobre recursos do Slint e avisos transitivos está em [`slint-research.md`](slint-research.md).
