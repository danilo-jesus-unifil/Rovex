# Documentação do Rovex

Este diretório reúne a documentação técnica do projeto por finalidade. O índice reduz a mistura entre auditorias, pesquisas, planos e relatórios de release e facilita localizar a evidência correta sem depender do histórico de commits.

| Pasta | Conteúdo | Exemplos principais |
|---|---|---|
| [`audits/`](audits/) | Auditorias de segurança, dependências, performance, UI e engenharia | [`ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md`](audits/ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md), [`FINAL-AUDIT-REPORT-2026-08-17.md`](audits/FINAL-AUDIT-REPORT-2026-08-17.md) |
| [`research/`](research/) | Pesquisas de plataforma, Slint, processos, conversões, preview e Shell | [`slint-research.md`](research/slint-research.md), [`open-with-research-2026-08-19.md`](research/open-with-research-2026-08-19.md) |
| [`plans/`](plans/) | Planos de implementação, refatoração e execução incremental | [`implementation-plan.md`](plans/implementation-plan.md), [`issue-2-execution-plan.md`](plans/issue-2-execution-plan.md) |
| [`reference/`](reference/) | Contratos operacionais, compatibilidade, dependências, testes e limitações conhecidas | [`testing.md`](reference/testing.md), [`compatibility-baseline.md`](reference/compatibility-baseline.md) |
| [`reports/`](reports/) | Relatórios gerais de implementação, modernização, estabilidade e UI | [`MODERNIZATION_REPORT.md`](reports/MODERNIZATION_REPORT.md), [`FINAL_STABILITY_REPORT.md`](reports/FINAL_STABILITY_REPORT.md) |
| [`releases/`](releases/) | Histórico de notas e relatórios por versão | [`RELEASE_REPORT-v0.1.15.md`](releases/RELEASE_REPORT-v0.1.15.md) |

Os poucos arquivos mantidos na raiz têm finalidade operacional ou convenção de plataforma: [`README.md`](../README.md) é a entrada do repositório, [`CHANGELOG.md`](../CHANGELOG.md) é o histórico curto, [`SECURITY.md`](../SECURITY.md) é a política reconhecida pelo GitHub e [`COMPATIBILITY.md`](../COMPATIBILITY.md) é incluído pelo pacote portable como matriz canônica de compatibilidade. Evidências Markdown de smoke tests ficam em [`../artifacts/validation/`](../artifacts/validation/), separadas da documentação normativa.

## Critério de manutenção

Novas auditorias devem ser salvas em `docs/audits/`, pesquisas e decisões externas em `docs/research/`, planos de execução em `docs/plans/`, contratos de operação em `docs/reference/`, relatórios consolidados em `docs/reports/` e notas específicas de versão em `docs/releases/`. Um arquivo só deve ir para a raiz quando for uma entrada do projeto, uma política reconhecida por ferramenta externa ou um arquivo distribuído deliberadamente no pacote portable.
