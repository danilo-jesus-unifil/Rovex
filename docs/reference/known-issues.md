# Limitações e problemas conhecidos

## Estado funcional atual

O Rovex já abre uma janela desktop Slint, carrega um diretório real em worker, mostra linhas de arquivos e diretórios, navega pelo endereço, sobe para a pasta pai, atualiza a listagem e exibe erro controlado para caminhos inválidos. O modo `--cli` permanece disponível para diagnóstico sem display.

## Operações

A exclusão é intencionalmente limitada a arquivos, links e diretórios vazios. Não há exclusão recursiva, pausa ou resolução interativa de conflitos. A UI agora dispara cópia, movimentação, renomeação e exclusão por uma modal de confirmação; cópia e movimentação aceitam múltiplos itens, recusam destinos existentes e executam em worker único com progresso e cancelamento cooperativo. A cópia entre volumes usa fallback verificado de copiar-e-remover; o cancelamento preserva a origem quando a cópia já terminou mas a remoção ainda não foi autorizada. Falhas por item são mostradas como resultado parcial e a pasta é recarregada após o worker.

## Interface

Há um filtro local sobre os itens da pasta atual, com fila latest-only e sem pesquisa recursiva. O histórico voltar/avançar, as abas reais com histórico independente, a seleção múltipla local por clique, Ctrl-clique, Shift-clique e Ctrl+A, o menu contextual e uma barra lateral com locais existentes já são funcionais e testados. A sidebar possui foco, setas e Enter/Space; estados vazios distinguem pasta vazia, filtro sem resultados e erro de filesystem. A barra lateral não analisa espaço, não enumera unidades inteiras e não cria favoritos persistentes. Ainda não há pesquisa global incremental, thumbnails, pré-visualização, drag and drop, tema configurável, atalhos completos, acesso à Lixeira ou integração com o Explorer do Windows. Esses recursos não são simulados e permanecem fora do escopo atual.

## Dependências

`cargo audit` não encontrou vulnerabilidades, mas reporta quatro avisos de crates sem manutenção (`bincode`, `paste`, `rustybuzz` e `ttf-parser`) introduzidos transitivamente por Slint 1.17.1. A base RustSec não indica atualização segura para essa cadeia durante esta verificação. O alerta permanece visível e está documentado em [`../research/slint-research.md`](../research/slint-research.md); trocar o toolkit ou aguardar uma cadeia upstream atualizada é uma decisão futura, não um problema ocultado por ignore.

## Windows 10/11

O projeto já produz um executável PE32+ x86-64 em build cruzado e a CI deve validar compilação e testes em `windows-latest`. A auditoria adicionou cobertura Linux para caminhos relativos, componentes pai symlink, nomes Unicode inválidos, publicação sem sobrescrita em corrida e mensagens humanizadas. Ainda permanecem pendentes a execução manual em Windows 10/11, paths longos, junctions e demais reparse points, UNC/SMB, permissões negadas, arquivos em uso, DPI por monitor, acessibilidade nativa do Windows, manifesto, instalador, assinatura e desinstalação.

Essas limitações permanecem gates explícitos para declarações de compatibilidade nativa completa no Windows e para futuras funcionalidades de distribuição; a v0.1.9 já foi publicada com o escopo e as limitações descritos neste documento.
