# ROVEX ENGINEERING AUDIT

**Issue de origem:** [#2 — uma prompt de ajuda](https://github.com/danilo-jesus-unifil/Rovex/issues/2)  
**Data:** 18 de agosto de 2026  
**Escopo desta etapa:** auditoria Foundation, sem implementação de features novas.

> **Conclusão executiva:** o Rovex possui uma fundação real, modular e testada para navegação local, seleção, abas, operações controladas, filtro local, menu contextual e conversões externas por workers. Ele ainda não é um Explorer completo e não deve declarar compatibilidade nativa completa com Windows 10/11 enquanto manifesto, execução nativa, DPI, acessibilidade, paths longos, UNC/SMB e reparse points não forem validados.

## 1. Current architecture

O projeto é um crate Rust 2024 com biblioteca `rovex_core`, binário `rovex`, `build.rs`, Slint 1.17.1 e backend Winit com renderer software. O fluxo de composição está em `src/desktop.rs`: a fachada cria `MainWindow`, resolve o caminho inicial, monta `AppContext`, registra handlers, inicia o carregamento e entra no event loop. A UI não acessa diretamente o filesystem nem subprocessos; os handlers delegam ao estado, schedulers e operações.

A arquitetura atual é compatível com o limite definido em `docs/architecture.md`: UI Slint → comandos/eventos → núcleo desktop → adapters de filesystem/operations/converters → workers. A separação recente em `desktop/context`, `desktop/handlers`, `desktop/jobs`, `desktop/state`, `operations` e `converters` reduziu arquivos grandes e manteve fronteiras de responsabilidade. A fachada pública em `src/lib.rs` reexporta os contratos de filesystem, operações, segurança e conversores.[1] [2]

## 2. Existing modules

| Área | Módulos principais | Responsabilidade observada |
|---|---|---|
| Entrada | `src/main.rs`, `src/lib.rs`, `build.rs` | CLI headless, composição pública, compilação Slint e ícone Windows |
| Filesystem | `src/filesystem.rs` | Listagem, classificação, metadados básicos, ordenação e erros humanizados |
| Segurança | `src/security.rs` | Validação de destinos, raízes, traversal, links e política de publicação |
| Operações | `src/operations/` | Cópia atômica, criação, renomeação, exclusão limitada, erros e testes |
| Conversores | `src/converters/` | Contratos, descoberta de backends, processos, pipes limitados e pipelines FFmpeg |
| Desktop | `src/desktop/` | Contexto, estado, navegação, listagem, seleção, view-model, handlers e jobs |
| UI | `ui/*.slint` | Tokens, dados, controles, toolbars, overlays e composição principal |
| Validação | `scripts/`, testes Rust e Xvfb | Smoke GUI, abas, menus, operações e conversão com diretórios separados |

Todos os módulos de produção Rust e Slint foram previamente modularizados para permanecer abaixo do limite objetivo de 400 linhas. A configuração de dependências mantém Slint restrito a `backend-winit-x11` no Linux de teste e usa `backend-winit` no Windows; `windows-sys` é target-specific.[3]

## 3. Existing features

A base atual lista diretórios reais sem seguir links simbólicos durante a enumeração, diferencia diretórios, arquivos e links, navega por caminho e pasta pai, mantém histórico por aba, atualiza a listagem, filtra a pasta atual com fila latest-only, oferece seleção por clique/Ctrl/Shift/Ctrl+A, abas independentes e sidebar de locais existentes.[4]

As operações visuais reais incluem copiar, mover, renomear e excluir com confirmação, worker, progresso, cancelamento cooperativo, resultado parcial e recarga verificada. A cópia usa publicação sem sobrescrita por padrão; movimentação possui fallback entre volumes; exclusão é limitada a arquivos, links e diretórios vazios. O menu contextual expõe as ações de arquivo e quatro conversões locais via FFmpeg/ffprobe, sem download de executáveis em runtime.[5]

A UI possui design system local escuro, tooltips customizados, tema consistente nas telas verificadas, estados de seleção/hover/foco, menu contextual, diálogo de operação, modo CLI e artefatos release Linux/Windows. A acessibilidade inclui labels, roles e a feature de acessibilidade do Slint, mas ainda precisa de validação nativa em Windows.

## 4. Missing features

As lacunas funcionais diretamente relacionadas ao objetivo do issue são clipboard integrado, arrastar e soltar, Lixeira/Recycle Bin, propriedades, navegação completa por teclado, arquivos ocultos configuráveis, ordenação escolhida pelo usuário, modos de visualização, pesquisa recursiva, filtros globais, índice opcional, thumbnails, painel de preview, metadados avançados, hashing, busca de duplicatas, análise de armazenamento, arquivos compactados, operações em lote, Open With, Terminal, integração Shell e distribuição instalável.[6]

Também faltam manifesto Windows efetivamente incorporado, validação nativa identificada em Windows 10 22H2 e Windows 11, DPI por monitor, alto contraste, leitor de tela, paths longos, UNC/SMB, junctions/reparse points, unidades removíveis, arquivos em uso, assinatura, instalador, desinstalação e atualização segura.[7]

## 5. Technical debt

A fundação possui documentação extensa e modularização recente, mas há divergências de manutenção: o README ainda descreve a release portable `v0.1.8`, enquanto o pacote e a release atual são `v0.1.9`; esse texto precisa ser reconciliado antes de uma nova distribuição. Alguns relatórios históricos descrevem baselines anteriores e devem ser claramente marcados como históricos para não confundirem o estado atual.

A camada de listagem materializa os metadados da pasta inteira antes de alimentar a representação visual. O `ListView` limita a representação renderizada, mas não elimina o custo de memória da preparação de diretórios extremos. A busca global, cache limitado e carregamento incremental ainda não existem. A descoberta de FFmpeg possui vários fallbacks e pipes limitados, porém ainda depende de executáveis locais e não há isolamento de processo equivalente a um sandbox.

A matriz de testes é forte para a primeira fatia, mas ainda depende de Linux/Xvfb para a maior parte dos testes gráficos. O cross-build Windows produz PE32+, mas não prova execução nativa. O branch principal contém o refinamento visual publicado, e a auditoria deve manter commits pequenos por feature nas próximas fases.

## 6. Security risks

Os riscos principais permanecem traversal e destinos ambíguos, links/junctions/reparse points, exclusão acidental, publicação concorrente, arquivos malformados, parsers/codecs vulneráveis, saturação de CPU/RAM/disco, DLL hijacking, command injection e artefatos de atualização sem integridade. A base já mitiga traversal, raízes, destinos existentes, links durante listagem, sobrescrita e comandos shell; falhas viram erros estruturados e resultados obsoletos são descartados por geração.[8]

O maior risco P0 de plataforma é a ausência de manifesto Windows efetivamente embutido, pois `asInvoker`, DPI awareness e long-path awareness permanecem documentados como pendentes, não comprovados. Outro risco importante é executar codecs externos sobre entradas não confiáveis: os argumentos são separados e os pipes têm leitura limitada, mas o processo ainda não possui sandbox de OS, job object, quota de CPU/memória ou política completa contra DLL hijacking no Windows.

Qualquer Lixeira, drag/drop, propriedades, preview, indexação, archive manager ou integração Shell deverá ser implementada como adapter/worker com limites e confirmação apropriada. Nenhuma feature destrutiva deve ser adicionada apenas por aparência.

## 7. Performance risks

A listagem de diretórios muito grandes ainda materializa metadados antes da apresentação. A busca recursiva futura poderá causar explosão de I/O, memória e resultados; por isso deverá ser cancelável, latest-only e limitada. Thumbnails, hashing, indexação e análise de armazenamento têm custos potencialmente altos e não podem executar no thread da UI.

A arquitetura atual já usa worker único para carregamento e filtro, snapshots compartilhados e descarte de resultados obsoletos. Os testes locais cobrem diretórios de até 100.000 itens no CLI e 10.000 itens na UI, mas ainda faltam benchmarks reproduzíveis de cold start, idle, busca recursiva, thumbnails, memória máxima e operações em arquivos grandes.[9]

## 8. Windows 10 risks

O Windows 10 22H2 x64 é o baseline de produto, mas não houve execução nativa nesta sessão. Permanecem sem prova DPI, acessibilidade, alto contraste, tema do sistema, long paths, UNC/SMB, reparse points, permissões, arquivos em uso, unidades removíveis, manifesto, instalador e desinstalador. APIs específicas do Windows 11 não devem vazar para o domínio sem fallback.

O próximo gate Windows 10 precisa capturar versão/build do sistema, verificar o manifesto no PE, executar a UI sem administrador, testar teclado e foco, operar em NTFS com paths longos e confirmar que falhas de permissão e arquivos em uso permanecem controladas.

## 9. Windows 11 risks

O Windows 11 tem compilação/CI documentada, mas a versão exata do runner não está fixada como matriz de produto. O risco é declarar suporte com base apenas em cross-build ou em um runner sem validar a experiência real do shell, DPI, alto contraste, menus, integração de caminhos e mudanças de APIs entre versões.

O próximo gate Windows 11 deve ser separado do Windows 10 e repetir o cenário de acessibilidade, DPI múltiplo, paths longos, UNC/SMB, reparse points, arquivos em uso e instalação. Recursos exclusivos do Windows 11 devem ser opcionais e possuir fallback explícito.

## 10. UI/UX gaps

A UI atual possui hierarquia escura consistente após o refinamento, porém ainda é uma primeira camada de Explorer. Faltam modos de visualização lista/detalhes/ícones, ordenação por nome/tipo/tamanho/data, indicador de seleção mais rico, propriedades, preview, drag/drop, clipboard visual, Lixeira, favoritos persistentes e pesquisa global. O cabeçalho de colunas existe visualmente, mas ainda não é interativo para ordenação.

A próxima melhoria deve priorizar operações que reduzem esforço real do usuário: navegação por teclado completa, ordenação, itens ocultos configuráveis, propriedades e clipboard seguro. A busca global e preview somente devem entrar depois dos contratos de cancelamento, limites e cache estarem definidos.

## 11. Accessibility gaps

A UI já declara labels, roles e tooltips para controles icon-only, usa foco de teclado em sidebar/listagem e mantém a feature de acessibilidade do Slint. Ainda não há evidência de leitor de tela, foco completo em todas as modais, navegação por teclado em todos os comandos, alto contraste nativo, escala 100–200%, dois monitores com DPI diferente ou validação no Windows.

A acessibilidade é critério de aceite, não polimento posterior. Cada nova view, menu contextual, diálogo, listagem, estado vazio e operação deve possuir foco previsível, nome acessível, feedback de erro e alternativa de teclado.

## 12. Testing gaps

Os 44 testes aprovados e 2 ignorados cobrem a fundação atual, mas não cobrem execução nativa Windows 10/11, manifesto, long paths NTFS, UNC/SMB, junctions/reparse points, arquivos em uso, permissões reais, unidades removíveis, disco cheio, cancelamento em arquivos maiores que 4 GB, drag/drop, clipboard, Lixeira, propriedades, ordenação, modos de visualização, pesquisa recursiva, preview ou instalador.

Também falta uma matriz de testes de regressão por feature, com teste focado, smoke funcional, check Windows e revisão de segurança. O teste visual automatizado de tema escuro é útil, mas não substitui inspeção de DPI, acessibilidade ou renderização nativa.

## 13. Documentation gaps

A documentação de arquitetura, testes, segurança, compatibilidade, dependências e limitações existe e é honesta. O principal gap imediato é reconciliar referências antigas à v0.1.8 no README e eventualmente em documentos de release após o refinamento visual posterior à tag v0.1.9. Também é necessário manter um roadmap vivo P0–P3 e registrar quais gates dependem de execução nativa Windows.

Cada feature futura deverá atualizar README, `docs/known-issues.md`, `docs/testing.md`, `COMPATIBILITY.md`, changelog e relatório técnico quando alterar o escopo distribuído.

## 14. Dependency risks

As dependências diretas são pequenas: Slint 1.17.1, slint-build 1.17.1, winres e windows-sys 0.61.2 somente no Windows. A escolha de backend X11 no Linux é específica do ambiente de smoke test; o Windows usa Winit próprio do target. A stack não é Linux-only.[3]

`cargo audit` não encontrou vulnerabilidades exploráveis, mas reporta quatro warnings transitivos de manutenção na cadeia do Slint: `bincode`, `paste`, `rustybuzz` e `ttf-parser`. `cargo deny` passa com a política atual. Esses warnings devem permanecer visíveis; não justificam trocar o toolkit sem uma correção upstream compatível, mas devem ser reavaliados em cada atualização de Slint.[10]

Não há downloads de executáveis em runtime. A descoberta de FFmpeg é ampla e controlada, mas qualquer distribuição futura de codecs deve documentar origem, hash, assinatura, licença, arquitetura e atualização.

## 15. Recommended implementation order

A ordem recomendada é:

| Prioridade | Próximo trabalho | Justificativa |
|---|---|---|
| P0 | Reconciliar documentação da versão; adicionar/verificar manifesto Windows `asInvoker` + DPI/long-path; criar gate nativo Windows 10/11 | Corrige risco de compatibilidade e evita declarações não comprovadas |
| P0 | Expandir testes de filesystem para long paths, reserved names, UNC/SMB, reparse points, arquivos em uso, permissões e cancelamento | Protege integridade antes de features novas |
| P1 | Navegação por teclado completa, ordenação, modos de visualização e itens ocultos configuráveis | Primeiro bloco de Core Explorer com baixo acoplamento e valor direto |
| P1 | Clipboard seguro, propriedades e Recycle Bin adapter | Recursos centrais, destrutivos e dependentes de Windows devem ser isolados e testados |
| P1 | Drag/drop com confirmação e operações canceláveis | Alta utilidade, mas exige contratos de origem/destino e segurança |
| P1 | Pesquisa recursiva cancelável e filtros globais | Só depois de limites de I/O, memória e latest-only estarem definidos |
| P2 | Thumbnails, preview, metadata e hashing | Custo de recursos e superfície de parsers exigem workers e limites |
| P2 | Duplicatas, storage analyzer, archives e batch operations | Ferramentas avançadas depois do core Explorer estável |
| P2 | Open With, Terminal e shell integration | Integração de plataforma e risco de execução de conteúdo não confiável |
| P3 | Instalador, assinatura, atualização e telemetry opt-in | Distribuição somente após gates nativos e supply chain |

## Roadmap por fases do issue

A **Foundation** está agora auditada. O próximo lote de código deve selecionar apenas um item P0, preferencialmente o manifesto Windows e seus testes de inspeção, ou a expansão de casos de filesystem que possa ser validada localmente. Depois virá o Core Explorer em commits separados, seguido por Search, Preview, Advanced Tools, Windows Integration e Distribution. Nenhuma fase deve ser implementada em lote único.

## Critérios de aceite da próxima etapa

A próxima alteração somente estará pronta quando houver implementação real, testes de regressão, validação de erro/cancelamento/recuperação, check Linux, check Windows quando pertinente, revisão de segurança, documentação atualizada e commit isolado. O branch de backup `backup/before-ui-polish-2026-08-18` permanece como recuperação para o refinamento visual anterior.

## Referências

[1]: docs/architecture.md "Rovex — Arquitetura técnica inicial"
[2]: src/lib.rs "Rovex core public facade"
[3]: Cargo.toml "Rovex manifest and target-specific dependencies"
[4]: README.md "Rovex current product status"
[5]: SECURITY.md "Rovex security policy and threat model"
[6]: docs/known-issues.md "Rovex known issues"
[7]: COMPATIBILITY.md "Rovex compatibility matrix"
[8]: docs/security-audit-research-notes-2026-08-17.md "Rovex security audit research notes"
[9]: docs/testing.md "Rovex testing strategy"
[10]: DEPENDENCIES.md "Rovex dependency inventory"
