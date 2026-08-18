# Pesquisa preliminar de drag-and-drop — 18/08/2026

A documentação oficial do Slint 1.17.1 foi consultada. O caminho presumido `/latest/docs/slint/guide/drag-and-drop/` retornou 404, portanto não deve ser tratado como API válida. A discussão oficial de lançamento do Slint 1.17 informa que a versão adiciona os blocos `DragArea` e `DropArea` e um tipo `DataTransfer`; isso precisa ser confirmado no source/API instalada antes da implementação.

A página oficial Rust `DataTransfer` documenta que o tipo representa múltiplas views dos dados de transferência, mas que atualmente o suporte comum inclui texto simples e imagem. A API expõe `plain_text()`, `image()`, `has_plain_text()`, `has_image()` e `user_data()`, e declara que a leitura pode ser lazy e não deve ser presumida barata. Não há método documentado nessa página para obter diretamente uma lista de caminhos de arquivos do Explorer, portanto um fallback de texto cru não deve ser aceito como path sem parsing e validação explícitos.

Consequentemente, o próximo trabalho deve começar pela inspeção do source/geração de bindings do Slint 1.17.1 e pela identificação de como o backend Winit Windows materializa arquivos arrastados. Se somente texto estiver disponível, o lote deve separar claramente: (a) ingestão de payload textual, (b) parsing de URI/file-list, (c) normalização/validação de caminhos absolutos e (d) operações reais sobre o scheduler. Nenhuma operação de cópia/movimentação deve ser disparada apenas por hover ou por payload não validado.

Referências:

[1]: https://docs.slint.dev/latest/docs/rust/slint/struct.DataTransfer.html — DataTransfer in slint, documentação Rust do Slint 1.17.1.
[2]: https://github.com/slint-ui/slint/discussions/12124 — Call for Testing for Slint 1.17, anúncio dos blocos de drag-and-drop.
[3]: https://docs.slint.dev/latest/docs/slint/reference/gestures/flickable/ — Flickable, referência usada no lote de Propriedades.

A discussão oficial do mantenedor do Slint confirma que 1.17 adiciona `DragArea`, `DropArea` e um tipo de transferência de dados, com a intenção explícita de testar arraste entre elementos e de/para outros aplicativos, incluindo desktop Windows. A página, porém, aponta para um guia cujo caminho histórico retornou 404 na versão atual; por isso a implementação deve depender do source/bindings da versão pinada no Cargo.lock e não de exemplos de uma versão nightly.

[4]: https://github.com/slint-ui/slint/discussions/12124 — Call for Testing for Slint 1.17, anúncio do mantenedor.

O guia oficial correto é `/latest/docs/slint/guide/development/drag-and-drop/`. Ele confirma que `DropArea` pode aceitar drops de outros aplicativos nas plataformas suportadas, porém o payload é um valor opaco `data-transfer` que deve ser construído/lido por callbacks no host. O exemplo usa callbacks host-side como `string-to-transfer(string) -> data-transfer`, `transfer-to-string(data-transfer) -> string` e `can-drop(data-transfer) -> bool`. `can-drop` roda durante hover e `dropped` no release; ambos retornam `DragAction`, e o retorno de `dropped` é reportado ao `drag-finished` da origem.

[5]: https://docs.slint.dev/latest/docs/slint/guide/development/drag-and-drop/ — Drag and Drop, guia oficial do Slint.
[6]: https://docs.slint.dev/latest/docs/slint/reference/drag-and-drop/droparea/ — DropArea, referência oficial.
[7]: https://docs.slint.dev/latest/docs/slint/reference/drag-and-drop/dragarea/ — DragArea, referência oficial.
