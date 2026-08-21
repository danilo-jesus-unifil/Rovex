# Relatório de release — Rovex v0.1.28

**Data:** 2026-08-20

A v0.1.28 fecha uma corrida residual descoberta durante a validação da reserva atômica de temporários. A v0.1.27 passou a criar um placeholder com `create_new(true)`, mas o pipeline o removia imediatamente antes do spawn do FFmpeg. Isso reabria a janela entre reserva e uso. Além disso, o FFmpeg era iniciado com `-n`, modo que recusa uma saída já existente, portanto a própria proteção introduzida impedia o caminho normal de execução.

| Item | Resultado |
|---|---|
| Versão | `0.1.28` |
| Checkpoint | `backup/before-temp-placeholder-use-20260820` |
| Código | `src/converters/pipeline.rs` e `src/converters/process.rs` |
| Regressões | `reserva_de_temporario_e_atomica_e_cria_placeholder` e `ffmpeg_pode_sobrescrever_placeholder_temporario_reservado` |
| Gate | `scripts/test_converter_temporary_contract.sh` |
| Suíte host | 109 testes aprovados; 2 ignorados explicitamente |

## Falha confirmada

A reserva atômica cria um arquivo vazio com nome exclusivo. Porém, a linha de retry removia esse arquivo antes de chamar `spawn_ffmpeg`. Após a remoção, outro worker poderia reservar o mesmo nome ou um processo externo poderia ocupar o caminho antes do FFmpeg. A lógica de publicação sem sobrescrita protegia somente o destino final, não o arquivo intermediário.

A primeira correção também mantinha `-n` no comando. Esse modo é adequado para não sobrescrever saídas existentes, mas é incompatível com o placeholder privado já criado pelo Rovex. O problema foi reproduzido com um backend fake que inspeciona os argumentos: somente o modo `-y` permite continuar quando o placeholder existe.

Esses pontos foram classificados como falhas reais porque foram observados no caminho efetivo e cobertos por testes executáveis, não inferidos apenas de documentação. O teste de reserva verifica exclusividade e existência do placeholder; o teste do backend verifica que o FFmpeg pode escrever sobre esse arquivo reservado.

## Correção implementada

O pipeline não remove mais o placeholder antes da tentativa. O arquivo reservado permanece durante o spawn, a validação e os retries; a limpeza continua no encerramento do pipeline ou em erro. O FFmpeg recebe `-y`, mas esse modo só é aplicado ao temporário privado, cujo nome foi reservado pelo Rovex. O destino final permanece protegido por `publish_file_no_replace`, que usa hard link/cópia sem sobrescrita.

A mudança separa as duas políticas: o temporário intermediário é uma área exclusiva e controlada pela operação; o destino do usuário nunca é sobrescrito. Erros de reserva diferentes de `AlreadyExists` continuam estruturados com operação, caminho, `ErrorKind` e código nativo.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| Reserva atômica | Aprovada com duas reservas distintas e placeholders reais |
| Uso do placeholder | Aprovado com backend fake que exige `-y` |
| Suíte host | 109 aprovados; 2 ignorados explicitamente |
| Repetição paralela | Dez rodadas completas passaram após a correção de fixtures |
| `cargo fmt` e `cargo check` | Aprovados |
| Clippy host/cross-Windows | Aprovado com `-D warnings` |
| Build release Windows GNU | Aprovado |
| Contrato de temporários | Aprovado; bloqueia remoção no retry e exige `create_new`/`-y` |
| Contratos anteriores | Ativação, Windows nativo, nomes reservados, descoberta FFmpeg e contenção aprovados |
| Auditoria/documentação | `cargo audit`, `cargo deny`, layout Markdown e links locais aprovados |

## Limitações honestas

A reserva e o placeholder reduzem a corrida entre workers do Rovex, mas não são sandboxing nem proteção por handle contra um processo externo que altere o arquivo depois do spawn. O FFmpeg ainda escreve por caminho. Uma futura proteção por handle/diretório deve ser projetada separadamente para Windows 10/11 e para os fallbacks de FFmpeg existentes.

Continuam pendentes autenticação por assinatura/hash, DLL hijacking, Job Objects em cenários de breakaway ou jobs aninhados, TOCTOU residual de origem/destino, ACLs, UNC/SMB, caminhos extended-length, arquivos bloqueados, disco cheio, DPI, acessibilidade e execução gráfica interativa completa em Windows 10/11.

## Referências

[1]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html "Rust standard library — OpenOptions"
[2]: https://ffmpeg.org/ffmpeg.html "FFmpeg documentation — Main options"
