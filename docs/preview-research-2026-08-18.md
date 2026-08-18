# Pesquisa de preview e thumbnails — 18/08/2026

## Fontes consultadas

A documentação do crate `image` descreve `ImageReader` como leitor multi-formato, com `with_guessed_format()` para inferir o formato pelo conteúdo em vez de confiar somente na extensão, e `decode()` para produzir uma imagem dinâmica.[1] A mesma documentação expõe `ImageDecoder::set_limits()` e informa que `Limits` possui limites de largura, altura e alocação; largura e altura são limites estritos, enquanto `max_alloc` é não estrito para alguns decodificadores.[2] O tipo `ImageDecoder` também oferece `total_bytes()`, que permite rejeitar imagens cujo buffer decodificado exceda o orçamento antes de materializar a imagem.[3]

A documentação da Microsoft sobre thumbnail providers informa que handlers de thumbnail normalmente executam em processo separado por padrão, que o Windows consulta um cache e que os tamanhos comuns de cache incluem 32, 96, 256 e 1024 pixels, embora os valores possam mudar.[4] Ela também orienta preservar a proporção e não aplicar overlays/adornos por conta própria, pois o sistema pode cuidar dessa apresentação.[4]

## Decisão para o Rovex

A primeira versão será um preview local somente para imagens estáticas comuns já cobertas pelo crate `image`, sem executar arquivos, sem usar shell e sem chamar handlers arbitrários do Explorer. O worker abrirá por conteúdo, aplicará limites de dimensão/alocação e produzirá uma imagem pequena em formato controlado. Arquivos sem suporte, corrompidos, acima dos limites ou potencialmente dinâmicos terão fallback para ícone genérico e mensagem controlada.

O cache será limitado por quantidade de entradas e bytes aproximados, com chave incluindo caminho, tamanho e `modified`/`created` disponíveis. O worker será cancelável por geração; resultados de preview antigos não poderão atualizar a UI. A primeira etapa não implementará PDF, vídeo, áudio, Office, codecs externos, Windows thumbnail providers ou execução de handlers COM, porque cada um possui superfície de ataque e compatibilidade própria. O preview nativo do Windows 10/11 será tratado como lote posterior e continuará explicitamente não declarado como concluído.

## Referências

[1]: https://docs.rs/image/latest/image/ "image crate — ImageReader e decodificação"
[2]: https://docs.rs/image/latest/image/struct.Limits.html "image::Limits — limites de recursos"
[3]: https://docs.rs/image/latest/image/trait.ImageDecoder.html "image::ImageDecoder — total_bytes e set_limits"
[4]: https://learn.microsoft.com/en-us/windows/win32/shell/thumbnail-providers "Microsoft Learn — Thumbnail Providers"
