# Problemas e limitações conhecidas

## Fundação

O repositório ainda não contém a interface desktop com Slint nem o manifesto Windows. O binário atual é um exercitador de desenvolvimento do núcleo e lista um diretório real no terminal. Não há abas, pesquisa, thumbnails, seleção visual, drag and drop, tema ou integração com o Explorer do Windows.

## Operações

A exclusão é intencionalmente limitada a arquivos, links e diretórios vazios. Não há exclusão recursiva, fila visual, pausa ou cancelamento porque essas operações ainda precisam de uma política de confirmação, detecção de links e relatório parcial. A cópia atual recusa destinos existentes; a resolução de conflitos será adicionada em uma camada de operações de maior nível.

## Windows

A compatibilidade efetiva com Windows 10/11 ainda não foi executada neste ambiente. Permanecem pendentes a validação de códigos Win32, paths longos, junctions, UNC/SMB, DPI por monitor, acessibilidade, arquivos em uso, manifesto, instalador, assinatura e desinstalação.

## Dependências e segurança

O núcleo atual não tem dependências externas de runtime, mas `cargo audit` e `cargo deny` ainda precisam ser executados em CI. Não há conversores, codecs, OCR ou parsers multimídia para auditar nesta fase. Nenhuma release estável deve ser publicada enquanto essas limitações forem relevantes para o escopo anunciado.
