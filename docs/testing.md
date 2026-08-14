# Estratégia de testes

## Ciclo local atual

O núcleo atual foi formatado e verificado com `cargo fmt --check`, testado com `cargo test --all-targets`, analisado com `cargo clippy --all-targets --all-features -- -D warnings`, compilado com `cargo build --release` e executado com `cargo run -- .`. O resultado observado foi de sete testes unitários aprovados, nenhum teste falho, Clippy sem diagnósticos e build release concluído.

## Cobertura da primeira fatia

Os testes atuais cobrem classificação de diretórios e arquivos, recusa de listagem sobre arquivo, recusa de destino existente por padrão, recusa de operação na raiz, cópia com arquivo temporário e validação do tamanho, renomeação, criação de diretório, exclusão de arquivo e exclusão de diretório vazio. A API de exclusão recursiva ainda não existe, deliberadamente.

## Próximos testes do filesystem

A próxima expansão deve incluir nomes Unicode e reservados do Windows, paths longos, arquivos maiores que 4 GB quando o runner permitir, links simbólicos e junctions, arquivos em uso, permissões negadas, colisões de nome, interrupção de cópia, disco cheio, unidades desconectadas e caminhos UNC. Cada bug encontrado deverá gerar um teste de regressão.

## Testes da interface e da distribuição

A camada desktop precisará ser verificada com teclado, foco, leitor de tela quando disponível, alto contraste, escalas de 100%, 125%, 150% e 200%, dois monitores com DPI diferente, tema claro/escuro, navegação de histórico, cancelamento e mensagens de erro. A distribuição exigirá instalação limpa, upgrade, reinstalação, versão portátil, assinatura, atualização verificada e desinstalação sem apagar documentos do usuário.

## Auditoria de dependências

A CI configura `cargo audit` e `cargo deny`. Os resultados desses comandos ainda não foram executados neste ambiente porque os binários não estavam instalados localmente; por isso, a primeira execução da pipeline é um critério de aceite pendente, não um resultado aprovado.
