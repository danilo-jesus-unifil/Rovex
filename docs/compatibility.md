# Compatibilidade

## Plataformas pretendidas

O alvo do produto é Windows 10 versão 22H2, OS build 19045, e Windows 11 em arquitetura x64. A fundação e a primeira UI também compilam em Linux para permitir testes unitários, smoke tests gráficos e validação do domínio, mas isso não constitui promessa de suporte de produto. A escolha do build 19045 é uma política de teste baseada na identificação oficial da Microsoft; a execução nativa ainda é necessária para provar compatibilidade.

| Ambiente | Estado atual | Observação |
|---|---|---|
| Linux x64 | Release nativo, CLI, UI em Xvfb e smoke test verificados | Não substitui testes de APIs e UX do Windows |
| Windows x64 GNU | Build release cruzado aprovado e artefato PE32+ validado | Execução nativa ainda pendente |
| Windows 10 22H2 x64 | Alvo mínimo escolhido; não executado nativamente nesta sessão | DPI, acessibilidade e filesystem ainda precisam de validação manual |
| Windows 11 x64 | Runner Windows do CI compila/testa; versão exata não é fixada no repositório | Deve ser testado separadamente de Windows 10 |
| Windows ARM64 | Fora do escopo inicial | Exigirá matriz e empacotamento próprios |

## Verificações atuais

O projeto compila com `cargo build --release --target x86_64-pc-windows-gnu`, e o CI possui job dedicado para esse cross-build. A UI foi executada em Xvfb com janela de 1100×720. O smoke test localizou a janela `Rovex`, editou a barra de endereço para `/tmp`, confirmou a navegação, mostrou listagem real por screenshot e encerrou sem crash. A matriz detalhada está em [`COMPATIBILITY.md`](../COMPATIBILITY.md).

## Regras de compatibilidade Windows

O aplicativo final deverá usar manifesto com execução `asInvoker`, DPI awareness declarado e suporte a caminhos longos conforme a documentação vigente da Microsoft. APIs exclusivas do Windows 11 deverão ficar atrás de uma camada de compatibilidade, com fallback ou desativação clara no Windows 10.

A validação final precisa ocorrer em máquinas Windows reais ou runners Windows. O fato de uma API compilar em Linux, ou de um binário funcionar no Windows 11, não demonstra compatibilidade com Windows 10.

## Itens ainda não verificados

Ainda não foram validados DPI por monitor, tema do sistema, múltiplos monitores, alto contraste, teclado, leitor de tela, unidades UNC/SMB, permissões específicas do Windows, reparse points, long paths, instalador, desinstalador, assinatura digital, associações de arquivos e comportamento com arquivos em uso. Esses pontos permanecem registrados como pendências, não como conformidade.
