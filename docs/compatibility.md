# Compatibilidade

## Plataformas pretendidas

O alvo do produto é Windows 10 e Windows 11 em arquitetura x64. A fundação atual também compila em Linux para permitir testes unitários do domínio e do filesystem, mas isso não constitui promessa de suporte de produto.

| Ambiente | Estado atual | Observação |
|---|---|---|
| Linux x64 | Verificado para núcleo e binário de desenvolvimento | Não substitui testes de APIs e UX do Windows |
| Windows 10 x64 | Ainda não verificado neste ambiente | Obrigatório antes da primeira release funcional |
| Windows 11 x64 | Ainda não verificado neste ambiente | Deve ser testado separadamente de Windows 10 |
| Windows ARM64 | Fora do escopo inicial | Exigirá matriz e empacotamento próprios |

## Regras de compatibilidade Windows

O aplicativo final deverá usar manifesto com execução `asInvoker`, DPI awareness declarado e suporte a caminhos longos conforme a documentação vigente da Microsoft. APIs exclusivas do Windows 11 deverão ficar atrás de uma camada de compatibilidade, com fallback ou desativação clara no Windows 10.

A validação final precisa ocorrer em máquinas Windows reais ou runners Windows. O fato de uma API compilar em Linux, ou de um binário funcionar no Windows 11, não demonstra compatibilidade com Windows 10.

## Itens ainda não verificados

Ainda não foram validados DPI por monitor, tema do sistema, múltiplos monitores, unidades UNC/SMB, permissões específicas do Windows, reparse points, long paths, instalador, desinstalador, assinatura digital, associações de arquivos e comportamento com arquivos em uso. Esses pontos permanecem registrados como pendências, não como conformidade.
