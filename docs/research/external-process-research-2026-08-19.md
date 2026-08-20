# Pesquisa de processos externos — 2026-08-19

## Evidências e decisão

A documentação do Rust descreve `Child::try_wait` como uma consulta não bloqueante ao status do subprocesso e recomenda aguardar o filho para liberar recursos do sistema; descartar `Child` sem garantir que terminou pode deixar processos ativos ou zombies [1]. O Rovex já usa `try_wait` em loop, `kill` no cancelamento/timeout e `wait` depois do término solicitado. A alteração deste lote deve preservar esse contrato, centralizando o cleanup para não esquecer leitores de stdout/stderr em nenhuma saída antecipada.

A documentação oficial do FFmpeg registra `-nostdin` como a opção para desabilitar interação no stdin, útil quando o processo executa em background [2]. O FFmpeg continua recebendo `-nostdin`. Durante a validação, o `ffprobe` disponível rejeitou essa opção; por isso ele usa `stdin(Stdio::null())` sem copiar uma flag exclusiva do FFmpeg. Essa combinação mantém ambos não interativos sem depender de compatibilidade incidental entre executáveis.

| Risco | Política adotada |
|---|---|
| Processo continua vivo depois de timeout/cancelamento | `kill`, `wait` e join dos leitores em caminho único de cleanup. |
| Deadlock no diagnóstico | stdout/stderr continuam em leitores dedicados e limitados a 64 KiB. |
| Processo espera entrada do usuário | `-nostdin` em FFmpeg; `Stdio::null()` em FFmpeg e ffprobe, sem stdin interativo. |
| Saída maliciosa ou excessiva | Leitura limitada; excesso vira erro, não cresce indefinidamente. |
| Arquivo temporário publicado após cancelamento | A pipeline verifica cancelamento antes de validar e publicar; erros removem o temporário. |
| Execução via shell/injeção | `Command::new` com `.arg`/`.args` separados e caminho de backend previamente resolvido. |

O timeout atual de cinco minutos é preservado neste lote para não mudar a política de produto sem dados de uso. O trabalho concreto é tornar o cleanup uniforme, adicionar `-nostdin` ao ffprobe e criar testes de processo fake que validem cancelamento e timeout com um executável controlado, sem depender de FFmpeg instalado.

## Referências

[1]: https://doc.rust-lang.org/std/process/struct.Child.html "Child in std::process — Rust documentation"

[2]: https://ffmpeg.org/ffmpeg.html "ffmpeg Documentation"
