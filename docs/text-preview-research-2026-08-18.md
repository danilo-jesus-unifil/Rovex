# Pesquisa de preview de texto — 18/08/2026

A recomendação W3C explica que BOM é uma assinatura no início do fluxo: UTF-8 pode conter `EF BB BF`, embora não precise de BOM, enquanto `FE FF` e `FF FE` indicam endianess de UTF-16.[1] A fonte também alerta que BOM pode causar caracteres invisíveis inesperados se não for removido/interpretado corretamente; por isso o preview deve consumir a assinatura antes de mostrar o conteúdo, sem alterar o arquivo.

A documentação da biblioteca padrão Rust informa que `str::from_utf8` valida os bytes e retorna erro quando a sequência não é UTF-8; não se deve usar a variante `from_utf8_unchecked` para conteúdo de usuário.[2] Isso permite uma política conservadora: ler apenas uma janela limitada, reconhecer BOM UTF-8/UTF-16LE/UTF-16BE, validar UTF-8 estrito ou rejeitar bytes inválidos/binários, sem aplicar heurística lossível como `from_utf8_lossy` para classificar binários.

## Decisão do lote

O preview de texto será limitado a 64 KiB por arquivo, exigirá arquivo regular não-symlink e verificará um prefixo de bytes antes de decodificar. UTF-8 sem BOM e UTF-8 com BOM serão aceitos; UTF-16LE/BE com BOM será aceito por uma conversão pequena e explicitamente limitada; bytes NUL e UTF-8 inválido sem BOM serão tratados como binários e receberão fallback. A leitura ocorrerá no PreviewScheduler existente, nunca na thread Slint, e o painel exibirá truncamento quando o arquivo exceder a janela. O lote não interpretará HTML, Markdown, JSON, scripts ou Office como código: mostrará somente texto literal sem links clicáveis, execução ou renderização.

## Referências

[1]: https://www.w3.org/International/questions/qa-byte-order-mark.en.html "W3C — Byte Order Mark"
[2]: https://doc.rust-lang.org/std/str/fn.from_utf8.html "Rust standard library — str::from_utf8"
