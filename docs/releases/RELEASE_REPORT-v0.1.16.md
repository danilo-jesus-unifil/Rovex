# Rovex v0.1.16 — Release report

## Resumo

A versão `v0.1.16` transforma a organização documental recém-concluída em uma regra verificável no CI e corrige uma falha real revelada pelo primeiro checkout limpo do novo gate. O lote não altera o comportamento do explorador; ele reduz regressões de manutenção e torna explícita a diferença entre evidência local ignorada e documentação versionada.

| Item | Resultado |
|---|---|
| Markdown permitido na raiz | `README.md`, `CHANGELOG.md`, `SECURITY.md` e `COMPATIBILITY.md` |
| Categorias versionadas | `docs/audits`, `docs/research`, `docs/plans`, `docs/reference`, `docs/reports` e `docs/releases` |
| Validador | `scripts/verify_markdown_layout.py` |
| Testes do validador | Quatro casos unitários em `scripts/test_verify_markdown_layout.py` |
| Job CI | `Documentation layout` em Ubuntu para push e pull request |
| Edge cases | `scripts/audit_edge_cases.sh` executado novamente com sete casos CLI |

## Investigação da falha

Antes de alterar o código, o estado foi auditado: `main` estava sincronizado com `origin/main` em `2ffebb3`, a release corrente era `v0.1.15`, o working tree estava limpo e o CI anterior havia passado. A auditoria encontrou uma inconsistência documental confirmada: `docs/reference/testing.md` ainda informava 46 testes, embora a suíte atual reportasse 97 aprovados e 2 ignorados. Também foi verificado o script `scripts/audit_edge_cases.sh`; ele existia e passou, portanto não foi recriado sem necessidade.

O primeiro CI do novo gate falhou em `Verify Markdown layout and links`. A causa não era uma falha do validador: `docs/README.md` continha um link formal para `../artifacts/validation/`. Esse diretório existe no sandbox quando capturas locais são geradas, mas é ignorado por `.gitignore` e não existe em um checkout limpo do GitHub. O diagnóstico foi confirmado comparando `git ls-files` e `git check-ignore`, e o link foi substituído por uma referência em código que explica a natureza local e não versionada dos artefatos.

> O primeiro resultado verde local não bastava: o checkout limpo do CI revelou que uma evidência gerada localmente estava sendo tratada indevidamente como documentação versionada.

## Implementação

O validador percorre Markdown fora de `.git` e `target`, extrai links Markdown e referências locais, ignora URLs externas, `mailto:` e fragmentos, resolve os destinos a partir do arquivo de origem e falha com diagnóstico quando um arquivo não existe. Em paralelo, verifica a política de raiz e rejeita qualquer `.md` novo fora da lista operacional permitida.

Os testes usam diretórios temporários e cobrem: um layout permitido com link existente; link local ausente; Markdown indevido na raiz; e links externos/mailto que não devem ser tratados como arquivos locais. O job CI executa o validador e os testes em checkout limpo, reduzindo o risco de depender de diretórios ignorados do ambiente de desenvolvimento.

A documentação foi alinhada: o guia de testes agora informa 97 testes aprovados e 2 ignorados explicitamente, descreve o novo gate e registra que `artifacts/validation/` é apenas uma área local de evidências. O README passou a listar os comandos do validador e do audit de edge cases.

## Validação

| Verificação | Resultado |
|---|---|
| `scripts/verify_markdown_layout.py` | Passou: 67 arquivos Markdown, zero violações na raiz e zero links quebrados |
| `python3 -m unittest scripts/test_verify_markdown_layout.py -v` | 4 testes passaram |
| `scripts/audit_edge_cases.sh` | Passou com sete casos CLI |
| `git diff --check` | Passou |
| `cargo fmt --all -- --check` | Passou |
| `cargo check --all-targets --all-features` | Passou |
| `cargo test --all-targets --all-features` | 97 passaram; 2 ignorados |
| CI inicial do gate | Falhou de forma útil pelo link para artefato ignorado |
| CI corretivo `8535469` | Documentation layout, audit, Ubuntu, Windows nativo e cross-Windows passaram |

## Limitações honestas

O gate verifica links locais e a organização dos Markdown versionados; ele não valida o conteúdo factual de cada relatório, links externos em tempo real, screenshots ignorados ou compatibilidade visual do Windows. O audit CLI cobre o contrato atual do modo `--cli`, mas não substitui a matriz nativa Windows 10/11 de Explorer, ACLs, DPI, acessibilidade, Shell, UNC/SMB e arquivos em uso. Os quatro warnings transitivos de manutenção da cadeia Slint continuam documentados e não foram ocultados.

## Referências

[1]: https://docs.github.com/en/actions "GitHub Actions documentation"

[2]: https://doc.rust-lang.org/cargo/commands/cargo-test.html "cargo test — The Cargo Book"
