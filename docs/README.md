# Documentação do Rovex

A documentação está separada por finalidade: auditorias, pesquisas, planos, referências, relatórios e releases.

| Pasta | Conteúdo | Exemplos principais |
|---|---|---|
| [`audits/`](audits/) | Auditorias de segurança, dependências, performance, UI e engenharia | [`ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md`](audits/ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md), [`FINAL-AUDIT-REPORT-2026-08-17.md`](audits/FINAL-AUDIT-REPORT-2026-08-17.md) |
| [`research/`](research/) | Pesquisas de plataforma, Slint, processos, conversões, preview e Shell | [`slint-research.md`](research/slint-research.md), [`open-with-research-2026-08-19.md`](research/open-with-research-2026-08-19.md) |
| [`plans/`](plans/) | Planos de implementação, refatoração e execução incremental | [`implementation-plan.md`](plans/implementation-plan.md), [`issue-2-execution-plan.md`](plans/issue-2-execution-plan.md) |
| [`reference/`](reference/) | Contratos operacionais, compatibilidade, dependências, testes e limitações conhecidas | [`testing.md`](reference/testing.md), [`compatibility-baseline.md`](reference/compatibility-baseline.md) |
| [`reports/`](reports/) | Relatórios gerais de implementação, modernização, estabilidade e UI | [`MODERNIZATION_REPORT.md`](reports/MODERNIZATION_REPORT.md), [`FINAL_STABILITY_REPORT.md`](reports/FINAL_STABILITY_REPORT.md) |
| [`releases/`](releases/) | Histórico de notas e relatórios por versão | [`RELEASE_REPORT-v0.1.15.md`](releases/RELEASE_REPORT-v0.1.15.md) |

Na raiz ficam os arquivos exigidos pelo projeto e pelas ferramentas: [`README.md`](../README.md) é a entrada, [`CHANGELOG.md`](../CHANGELOG.md) é o histórico, [`SECURITY.md`](../SECURITY.md) é a política reconhecida pelo GitHub e [`COMPATIBILITY.md`](../COMPATIBILITY.md) é a matriz incluída no pacote portable. Evidências Markdown de smoke tests ficam em `artifacts/validation/`; o diretório é ignorado pelo Git e não é documentação versionada.

## Critério de manutenção

Salve novas auditorias em `docs/audits/`, pesquisas e decisões externas em `docs/research/`, planos em `docs/plans/`, contratos operacionais em `docs/reference/`, relatórios consolidados em `docs/reports/` e notas de versão em `docs/releases/`. Use a raiz somente para a entrada do projeto, políticas reconhecidas por ferramentas externas ou arquivos distribuídos deliberadamente no pacote portable.
