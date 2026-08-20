# Nomes reservados de dispositivos Windows no Rovex

Data: 2026-08-20

## Investigação

O roadmap mantinha nomes reservados do Windows como risco pendente. A auditoria encontrou um caso específico no predicado `is_reserved_windows_name`: ele rejeitava `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `COM1`–`COM9` e `LPT1`–`LPT9`, mas usava `is_ascii_digit()` para as portas. Com isso, `COM¹`, `COM²`, `COM³`, `LPT¹`, `LPT²` e `LPT³` não eram reconhecidos pela regra interna.

A documentação oficial de nomes de arquivos lista explicitamente os seis nomes com dígitos sobrescritos como reservados e explica que o Windows reconhece ¹, ² e ³ como dígitos de dispositivos. Ela também determina que a reserva continua válida quando o nome recebe uma extensão, como `COM².txt`.[1]

## Correção

O predicado Windows passou a usar uma lista explícita dos nomes ASCII e sobrescritos. A comparação continua case-insensitive para o prefixo ASCII via `to_ascii_uppercase`, remove espaços e pontos finais conforme a política existente e separa o primeiro componente antes de uma extensão. A regra continua compilada apenas em Windows e não altera o comportamento Unix.

## Validação

| Validação | Cobertura |
|---|---|
| Teste Windows nativo | `create_directory` rejeita nomes ASCII, sobrescritos e extensões |
| Gate estrutural | Exige os seis nomes no predicado, na fixture e no workflow |
| Host | Check, testes, Clippy e diff |
| Cross-Windows GNU | Check, Clippy e build release |
| CI | `cargo test` em `windows-latest` executa a fixture real |

O gate `scripts/test_reserved_windows_names_contract.sh` não tenta criar esses nomes em Linux e não declara a semântica Windows a partir de um cross-build isolado; ele apenas impede que a cobertura nativa seja removida silenciosamente.

## Limitações

O lote cobre a criação de diretórios via operação interna. Ainda são necessários fixtures Windows para nomes inválidos com caracteres de controle, streams alternativos, namespaces `\\?\\`, UNC, ACLs e diferenças entre Shell e APIs de filesystem.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file "Naming Files, Paths, and Namespaces"
