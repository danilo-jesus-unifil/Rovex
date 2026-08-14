# Pesquisa de interface desktop — Slint

## Fontes consultadas

A página do crate Slint no crates.io foi consultada em 14/08/2026 após o resultado de busca indicar `slint` v1.17.1 e MSRV 1.92.0. A página dinâmica não apresentou texto completo no navegador, portanto a versão deve ser confirmada pelo próprio Cargo durante a resolução do lockfile.

Fonte: https://crates.io/crates/slint

O anúncio oficial do Slint 1.17 informa que a versão foi publicada como uma etapa para tornar o toolkit mais adequado a desktop e destaca drag and drop, ícone de bandeja, tooltips, grupos de rádio e bindings bidirecionais em linhas de modelos. O toolkit é descrito como escrito em Rust, com APIs para Rust, C++, JavaScript e Python, destinado a aplicações desktop, embedded e mobile.

Fonte: https://slint.dev/blog/slint-1.17-released

## Decisão

A primeira interface será deliberadamente pequena: janela única, barra de endereço, lista real do diretório atual, botão de atualização, navegação para diretório pai e mensagens de erro. A UI não deverá executar operações de filesystem no thread visual. A dependência Slint será adicionada com versão exata resolvida pelo Cargo e o lockfile será auditado com cargo-audit e cargo-deny.

## Risco identificado

A versão atual do Slint exige Rust mínimo mais novo que o MSRV histórico do núcleo. O projeto já usa Rust 1.97.1 no `rust-toolchain.toml`, portanto a dependência é compatível com o toolchain fixado, mas o campo `rust-version` do pacote deverá ser atualizado para refletir a realidade quando o crate for adicionado.

## APIs de integração confirmadas

A documentação do `Weak` confirma que `upgrade_in_event_loop` recebe uma closure `Send + 'static`, agenda a execução na fila interna e a executa na próxima iteração do event loop; se o componente já tiver sido destruído, a closure não será chamada.

A documentação do `ModelRc` confirma que o tipo não é `Send` e deve ser manipulado no thread principal. Dados produzidos em outra thread devem ser enviados de volta pelo `Weak::upgrade_in_event_loop` ou `invoke_from_event_loop`, onde o `VecModel` pode ser atualizado com segurança.

Fontes: https://docs.slint.dev/latest/docs/rust/slint/struct.Weak e https://docs.slint.dev/latest/docs/rust/slint/struct.ModelRc

## Auditoria de dependências e licenças

O cargo-deny documenta que identificadores customizados podem ser declarados com o prefixo `LicenseRef-` na lista de licenças permitidas. A política do Rovex agora permite somente as licenças SPDX observadas na árvore e as duas referências customizadas declaradas pelo Slint: `LicenseRef-Slint-Royalty-free-2.0` e `LicenseRef-Slint-Software-3.0`.

O cargo-audit encontrou quatro advisories de manutenção, não vulnerabilidades exploráveis: `bincode` 2.0.1, `paste` 1.0.15, `rustybuzz` 0.20.1 e `ttf-parser` 0.25.1. O cargo-audit termina com código 0, mas o cargo-deny trata advisories de manutenção como erro por padrão. Como esses crates são transitivos do Slint e a base RustSec informa que não há upgrade seguro disponível para o conjunto atual, a configuração deverá usar `unmaintained = "workspace"` ou `"warn"` conforme a política suportada, mantendo os avisos visíveis e documentados em vez de ignorá-los.

Fontes: https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html e https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html
