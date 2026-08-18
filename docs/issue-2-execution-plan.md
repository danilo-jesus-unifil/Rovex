# Plano de execução do issue #2

**Issue:** [#2 — uma prompt de ajuda](https://github.com/danilo-jesus-unifil/Rovex/issues/2)  
**Estado:** aberto  
**Método:** executar uma fase por vez, validar e registrar evidências antes de avançar.

## Regras que governam toda a execução

O issue define o Rovex como um filesystem client desktop para Windows 10/11, não apenas como uma interface. A prioridade obrigatória é segurança, integridade dos dados, estabilidade, correção, acessibilidade, desempenho, UX e somente depois novas features. Nenhuma implementação deve substituir funcionalidades reais por mocks, duplicar subsistemas existentes sem necessidade ou declarar uma feature pronta sem testes e evidências.

A UI deve emitir comandos/eventos; o core deve decidir; adapters devem executar; workers devem processar; e os resultados devem voltar como eventos tipados. Cada nova feature deverá responder como funciona, como falha, como cancela, como recupera, como é testada, como se comporta no Windows 10/11, qual custo de CPU/memória e se pode gerar corrupção, race condition, bloqueio da UI ou resultado obsoleto.

## Fases do master prompt

| Fase | Escopo do issue | Critério para avançar |
|---|---|---|
| 0 | Ler o issue, preservar o contexto e decompor o trabalho | Plano registrado e nenhuma implementação prematura |
| 1 — Foundation | Auditar arquitetura, filesystem, operações, UI, workers, testes e compatibilidade Windows; corrigir problemas encontrados | Auditoria `ROVEX ENGINEERING AUDIT` completa, roadmap P0–P3 e validações da base |
| 2 — Core Explorer | Clipboard, drag/drop, recycle bin, propriedades, navegação por teclado, ocultos, ordenação e modos de visualização | Cada feature isolada, implementada sobre módulos existentes, testada e documentada |
| 3 — Search | Busca recursiva, filtros, engine de busca e índice opcional | Busca cancelável, sem travar UI, com limites e testes de caminhos/erros |
| 4 — Preview | Thumbnails, painel de preview, metadados e hashing | Parsers seguros, limites de memória/tempo, sem execução de conteúdo não confiável |
| 5 — Advanced Tools | Duplicatas, análise de armazenamento, arquivos compactados e operações em lote | Operações controladas, canceláveis e com tratamento de integridade |
| 6 — Windows Integration | Open With, Terminal, Shell integration, Recycle Bin e integração contextual | Check Windows 10/11 e adapters isolados de plataforma |
| 7 — Distribution | Installer, portable, assinatura, atualização e pipeline de release | Artefatos verificáveis, checksums, documentação e release reproduzível |

## Próxima etapa isolada

A próxima etapa é exclusivamente a **Fase 1 — Foundation / auditoria**. Antes de escrever código, serão lidos README, `docs/architecture.md`, `docs/implementation-plan.md`, `docs/testing.md`, `COMPATIBILITY.md`, `DEPENDENCIES.md`, relatórios existentes e a árvore do projeto. Em seguida será produzido o formato exigido pelo issue:

```text
ROVEX ENGINEERING AUDIT
1. Current architecture
2. Existing modules
3. Existing features
4. Missing features
5. Technical debt
6. Security risks
7. Performance risks
8. Windows 10 risks
9. Windows 11 risks
10. UI/UX gaps
11. Accessibility gaps
12. Testing gaps
13. Documentation gaps
14. Dependency risks
15. Recommended implementation order

ROADMAP
P0 — Critical
P1 — High
P2 — Medium
P3 — Experimental
```

Nenhuma feature da Fase 2 ou posterior será marcada como iniciada antes de essa auditoria ser concluída e validada. Problemas P0 descobertos na auditoria serão tratados em mudanças pequenas, com branch/commit próprio e validação incremental.

## Workflow de cada lote

Cada lote seguirá o ciclo: inspecionar o módulo existente; definir risco e critério de aceite; implementar uma mudança pequena; executar `cargo fmt`, `cargo check`, testes focados e validações de plataforma pertinentes; revisar segurança, concorrência, desempenho, UX e acessibilidade; atualizar documentação; criar commit descritivo; e somente então avançar.

## Critérios de pronto

Uma feature só será considerada pronta quando houver implementação real, fluxo de erro estruturado, cancelamento quando aplicável, recuperação ou comportamento seguro, testes relevantes, documentação atualizada, validação Windows 10/11 quando aplicável e evidência de que resultados obsoletos não substituem o estado atual.
