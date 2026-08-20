# Rovex v0.1.12 — Release report

## Resumo

A versão `v0.1.12` endurece a fronteira entre o Rovex e os conversores externos FFmpeg/ffprobe. O worker continua sem shell, usa argumentos separados, fecha stdin explicitamente e garante cleanup do subprocesso em cancelamento, timeout e erros de espera. A release também corrige o smoke de JPEG XL, que estava clicando em Propriedades por usar uma coordenada antiga do menu contextual, e remove uma opção `-nostdin` incompatível com o ffprobe disponível.

| Item | Valor |
|---|---|
| Versão | `0.1.12` |
| Base de distribuição | Portable Windows v0.1.11 permanece disponível como ZIP verificado |
| Timeout de conversão | Cinco minutos em produção; duração parametrizada nos testes |
| Diagnóstico | stdout/stderr em leitores dedicados, limite de 64 KiB |
| Cancelamento | `kill` + `wait` + join dos leitores antes de retornar |
| Entrada do processo | `Stdio::null()` para FFmpeg e ffprobe; `-nostdin` apenas no FFmpeg |
| Assinatura | Os artefatos continuam não assinados |

## Correções verificadas

A conversão real de uma imagem PNG para JPEG XL voltou a publicar `entrada.jxl` com 67 bytes no smoke gráfico. O diálogo de erro controlado continua sendo usado quando o backend não valida a saída; não há publicação parcial. A captura que revelou o problema registrou a mensagem do ffprobe: `Failed to set value '-select_streams' for option 'nostdin': Option not found`. A correção foi específica: retirar a flag do ffprobe, manter stdin nulo e conservar `-nostdin` no FFmpeg conforme a documentação do próprio FFmpeg.

Os testes fake usam executável controlado, verificam cancelamento rápido, timeout sem esperar os 30 segundos naturais e argumentos separados. O fixture é publicado por rename atômico para não introduzir `ETXTBSY` em testes paralelos. O stress de 20 rodadas passou.

## Validação

Passaram `cargo fmt --all -- --check`, `cargo test --all-targets --all-features` com 90 testes aprovados e 2 ignorados documentados, Clippy estrito no host e em `x86_64-pc-windows-gnu`, `cargo audit`, `cargo deny check advisories licenses bans sources`, `cargo check`/`cargo build --release` Windows, verificação do manifesto PE e os smoke tests de UI já existentes. O smoke JXL foi repetido após `cargo build --release` nativo e criou a saída real; os testes de tabs, atalhos, ordenação, ocultos, nova pasta, clipboard, propriedades, busca, preview e settings permanecem parte da matriz.

A execução nativa em Windows 10/11, SmartScreen, assinatura Authenticode, ACLs, DPI, UNC/SMB e acessibilidade nativa ainda exigem uma máquina ou runner Windows real. O cross-build e o manifesto não substituem essa validação.

## Referências

A política de processo e as fontes oficiais estão em [`../research/external-process-research-2026-08-19.md`](../research/external-process-research-2026-08-19.md). A distribuição portable, o checksum e as limitações de assinatura estão em [`./RELEASE_REPORT-v0.1.11.md`](././RELEASE_REPORT-v0.1.11.md) e [`../research/distribution-research-2026-08-19.md`](../research/distribution-research-2026-08-19.md).
