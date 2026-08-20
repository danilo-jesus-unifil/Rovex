# Rovex v0.1.22 — nomes reservados Windows sobrescritos

Data: 2026-08-20

A v0.1.22 corrige uma lacuna real no contrato de criação de diretórios Windows. O predicado anterior reconhecia nomes reservados ASCII, mas não reconhecia os dígitos sobrescritos ¹, ² e ³ usados pelo Windows nos aliases de dispositivos `COM` e `LPT`.

| Item | Resultado |
|---|---|
| Versão | `0.1.22` |
| Branch de backup | `backup/before-reserved-device-names-20260820` |
| Código | `src/security.rs` |
| Fixture nativa | `src/operations/tests.rs` sob `#[cfg(windows)]` |
| Gate estrutural | `scripts/test_reserved_windows_names_contract.sh` |
| Pesquisa | `docs/research/reserved-device-names-2026-08-20.md` |

## Falha confirmada

A função `is_reserved_windows_name` removia espaços e pontos finais, separava a extensão e convertia o prefixo ASCII para maiúsculas. Para portas, porém, ela aceitava somente um quarto byte ASCII entre `1` e `9`. Assim, `COM¹`, `COM²`, `COM³`, `LPT¹`, `LPT²` e `LPT³` não eram rejeitados pela política interna.

A documentação oficial de Naming Files, Paths, and Namespaces lista os seis nomes sobrescritos como reservados, diz que a reserva se aplica a arquivos e diretórios e inclui extensões como `COM².txt`. A mesma fonte explica que o Windows reconhece ¹, ² e ³ como dígitos válidos nos nomes de dispositivos.[1]

> A lacuna foi confirmada por inspeção do predicado e não por uma hipótese sobre o filesystem. A execução nativa Windows é usada para provar que a operação real continua recusando os nomes.

## Correção

O predicado Windows passou a usar uma lista explícita para os nomes `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `COM1`–`COM9`, `COM¹`–`COM³`, `LPT1`–`LPT9` e `LPT¹`–`LPT³`. O stem continua sendo extraído antes da extensão e o trim de espaços/pontos finais permanece. A regra é compilada somente em Windows.

Não foi adicionado fallback para shell, renomeação automática ou criação de nomes alternativos. A operação continua recusando um destino inválido com `ValidationError::InvalidPath` e a mensagem `nome reservado do Windows`.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| Testes host | 104 aprovados; 2 ignorados explicitamente |
| Testes focados | Segurança e operações passaram em modo single-thread |
| Fixture Windows | Inclui ASCII, seis sobrescritos e `COM².txt`/`LPT².txt` |
| Gate estrutural | Predicado, fixture e workflow exigidos por script |
| Check/Clippy host | Aprovados sem warnings |
| Check/Clippy/build Windows GNU | Executados antes do CI |
| CI Windows | `cargo test` executa a fixture `#[cfg(windows)]` nativamente |
| Documentação | Layout e links locais verificados |

O teste host não tenta validar semântica Windows que não existe no Linux. O cross-build comprova compilação, enquanto a confirmação comportamental fica a cargo do job `windows-latest`.

## Limitações honestas

O lote cobre nomes de dispositivos reservados no caminho final de criação de diretório. Ainda são necessários fixtures nativos para caracteres de controle, streams alternativos, namespaces `\\?\\`, UNC, ACLs, arquivos em uso, mounted folders e outras tags de reparse. O Windows Shell e APIs de filesystem podem ter diferenças adicionais; nenhuma delas é marcada como resolvida nesta release.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file "Naming Files, Paths, and Namespaces"
