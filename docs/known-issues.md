# Limitações e problemas conhecidos

## Estado funcional atual

O Rovex já abre uma janela desktop Slint, carrega um diretório real em worker, mostra linhas de arquivos e diretórios, navega pelo endereço, sobe para a pasta pai, atualiza a listagem e exibe erro controlado para caminhos inválidos. O modo `--cli` permanece disponível para diagnóstico sem display.

## Operações

A exclusão é intencionalmente limitada a arquivos, links e diretórios vazios. Não há exclusão recursiva, fila visual, pausa, cancelamento ou resolução de conflitos porque essas operações ainda precisam de uma política de confirmação, detecção de links e relatório parcial. A cópia de núcleo recusa destinos existentes e protege a publicação contra uma criação concorrente; a UI ainda não dispara cópia, mover ou exclusão. A resolução de conflitos será adicionada em uma camada de operações de maior nível.

## Interface

Há um filtro local sobre os itens da pasta atual, com fila latest-only e sem pesquisa recursiva. O histórico voltar/avançar, a seleção múltipla local por clique, Ctrl-clique, Shift-clique e Ctrl+A e uma barra lateral com locais existentes já são funcionais e testados. A sidebar possui foco, setas e Enter/Space; estados vazios distinguem pasta vazia, filtro sem resultados e erro de filesystem. A barra lateral não analisa espaço, não enumera unidades inteiras e não cria favoritos persistentes. Ainda não há pesquisa global incremental, abas, thumbnails, pré-visualização, drag and drop, menu contextual, tema configurável, atalhos completos, integração com o Explorer do Windows ou operações de arquivo disparadas pela UI. Esses recursos não são simulados e permanecem fora da primeira fatia.

## Dependências

`cargo audit` não encontrou vulnerabilidades, mas reporta quatro avisos de crates sem manutenção (`bincode`, `paste`, `rustybuzz` e `ttf-parser`) introduzidos transitivamente por Slint 1.17.1. A base RustSec não indica atualização segura para essa cadeia durante esta verificação. O alerta permanece visível e está documentado em [`docs/slint-research.md`](slint-research.md); trocar o toolkit ou aguardar uma cadeia upstream atualizada é uma decisão futura, não um problema ocultado por ignore.

## Windows 10/11

O projeto já produz um executável PE32+ x86-64 em build cruzado e a CI deve validar compilação e testes em `windows-latest`. A auditoria adicionou cobertura Linux para caminhos relativos, componentes pai symlink, nomes Unicode inválidos, publicação sem sobrescrita em corrida e mensagens humanizadas. Ainda permanecem pendentes a execução manual em Windows 10/11, paths longos, junctions e demais reparse points, UNC/SMB, permissões negadas, arquivos em uso, DPI por monitor, acessibilidade nativa do Windows, manifesto, instalador, assinatura e desinstalação.

Nenhuma release estável deve ser publicada enquanto essas limitações forem relevantes para o escopo anunciado.
