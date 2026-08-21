# Relatório de release — Rovex v0.1.27

**Data:** 2026-08-20

A v0.1.27 corrige uma corrida real na reserva dos temporários de conversão. O pipeline consultava `candidate.exists()` e depois retornava um caminho ainda não reservado. Em duas conversões concorrentes com a mesma origem e tipo, ambas poderiam observar o mesmo nome livre e iniciar FFmpeg no mesmo arquivo temporário. A publicação final sem sobrescrita continuava protegida, mas a etapa intermediária podia compartilhar ou sobrescrever conteúdo.

| Item | Resultado |
|---|---|
| Versão | `0.1.27` |
| Checkpoint | `backup/before-temp-reservation-hardening-20260820` |
| Código | `src/converters/paths.rs` |
| Regressão | `reserva_de_temporario_e_atomica_e_cria_placeholder` |
| Pesquisa | `docs/research/process-containment-research-2026-08-20.md` |
| Contagem prevista | 108 testes aprovados; 2 ignorados explicitamente |

## Falha confirmada

A função `temporary_path` gerava nomes a partir de PID, timestamp e tentativa. A verificação `exists()` não era uma reserva: outro worker ou processo podia criar o mesmo caminho imediatamente depois da consulta. A existência de `create_new(true)` na publicação do destino não protegia a etapa anterior, na qual o FFmpeg escrevia o temporário.

A falha foi classificada como corrida real de filesystem porque a separação entre consulta e uso está no caminho efetivo de conversão, e a solução não depende de aumentar a precisão do relógio. O teste cria duas reservas para o mesmo destino e exige caminhos diferentes, verificando também que cada reserva deixa um placeholder real antes do spawn.

> A correção remove a janela `exists`→uso; ela não transforma o temporário em armazenamento transacional contra alterações externas posteriores.

## Correção implementada

`temporary_path` agora tenta abrir cada candidato com `OpenOptions::new().write(true).create_new(true)`. Quando a abertura succeeds, o arquivo vazio é fechado e o caminho reservado é retornado para o FFmpeg. Quando o nome já existe, a função avança para a próxima tentativa; outros erros de I/O são convertidos em `OperationError::FileSystem`, preservando operação, caminho, `ErrorKind` e código nativo.

O pipeline mantém a limpeza em erro/cancelamento e remove o placeholder antes de cada tentativa de backend. Como a reserva já foi feita com `create_new`, conversões concorrentes não compartilham a mesma entrada temporária por uma observação simultânea de “não existe”. A publicação final continua usando hard link/cópia sem sobrescrita.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| Teste da reserva atômica | Aprovado: duas reservas distintas e dois placeholders existentes |
| Suíte host | 108 aprovados; 2 ignorados explicitamente |
| Repetição paralela | 10 rodadas completas passaram depois da correção anterior de fixtures |
| `cargo fmt` | Aprovado |
| Clippy host | Aprovado com `-D warnings` |
| Check/Clippy Windows GNU | Aprovados com `-D warnings` |
| Build release Windows GNU | Aprovado |
| Contratos de segurança | Ativação, caminhos Windows, nomes reservados, descoberta FFmpeg e contenção aprovados |
| Auditoria/documentação | `cargo audit`, `cargo deny`, layout Markdown e links locais aprovados |

## Limitações honestas

A reserva atômica evita a corrida entre “consultar” e “criar”, mas não impede que outro processo apague ou substitua o caminho depois que a reserva foi fechada, nem substitui ACLs, handles seguros ou isolamento de diretório. O FFmpeg ainda escreve por caminho; uma validação por handle ou diretório seguro seria uma etapa posterior e precisa respeitar a compatibilidade Windows 10/11.

Continuam pendentes autenticação por assinatura/hash dos backends, DLL hijacking, Job Objects em cenários de breakaway ou jobs aninhados, TOCTOU residual de origem/destino, ACLs reais, UNC/SMB, caminhos extended-length, arquivos bloqueados, disco cheio, DPI, acessibilidade e execução gráfica interativa completa em Windows 10/11.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights "Microsoft — File Security and Access Rights"
[2]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html "Rust standard library — OpenOptions"
