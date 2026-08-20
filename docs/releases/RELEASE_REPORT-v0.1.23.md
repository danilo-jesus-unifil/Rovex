# Relatório de release — Rovex v0.1.23

**Data:** 2026-08-20

A v0.1.23 é uma release de auditoria e endurecimento do núcleo. A investigação do estado v0.1.22 confirmou quatro inconsistências reais envolvendo busca, saídas Windows, pais reparse e exclusão de junctions. A mesma rodada confirmou um risco de confiança no diretório atual usado durante a descoberta de FFmpeg/ffprobe; esse risco foi documentado, mas não recebeu uma alteração especulativa.

| Item | Resultado |
|---|---|
| Versão | `0.1.23` |
| Branch de backup | `backup/before-audit-hardening-20260820` |
| Código | `src/search/`, `src/converters/paths.rs`, `src/security.rs`, `src/operations/entry.rs` |
| Regressões | Busca em raiz symlinkada, colisão de caixa Windows, pai junction Windows e classificação de reparse na exclusão |
| Auditoria | `docs/research/audit-2026-08-20.md` |
| Pesquisa residual | `artifacts/validation/audit-executable-search-2026-08-20.md` e `audit-risk-research-2026-08-20.md` |

## Falhas confirmadas

A busca recursiva protegia subdiretórios reparse, mas não rejeitava explicitamente uma raiz reparse antes de `read_dir`. A comparação de origem e saída usava igualdade lexical, insuficiente para o filesystem case-insensitive do Windows. A validação de destinos reconhecia symlink nos pais, mas não o conjunto de reparse points representado por `FILE_ATTRIBUTE_REPARSE_POINT`. Por fim, a exclusão podia observar um junction final como diretório e entrar no caminho de inspeção de conteúdo.

O risco de descoberta de FFmpeg foi confirmado por inspeção do código e da documentação oficial: o diretório de trabalho está entre candidatos e as APIs Windows documentam que ele pode preceder o PATH dependendo do modo de busca.[1] [2] Isso não é execução automática, pois a conversão exige uma ação explícita do usuário, mas significa que a origem do backend não é autenticada por hash, assinatura ou diretório confiável. A decisão foi mantida fora desta release para não quebrar overrides e instalações válidas sem um contrato adversarial nativo.

## Correções

A busca ganhou `SearchError::RootRedirected` e validação da raiz antes da travessia. A saída de conversão passou a canonicalizar caminhos existentes no Windows para detectar colisões de caixa. A política de destinos usa um helper multiplataforma e recusa reparse points em todos os componentes pais antes da normalização. A exclusão Windows verifica o atributo reparse do caminho final e o encaminha para o ramo de link/arquivo, evitando `read_dir` ou `ensure_directory_empty` sobre junctions.

Nenhum fallback de shell foi adicionado, nenhuma conversão baixa executáveis e nenhuma exclusão permanente foi introduzida no Windows. A Lixeira permanece o adapter de exclusão Windows; as mudanças apenas impedem que um reparse point seja tratado como diretório comum.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| Testes host | 105 aprovados; 2 ignorados explicitamente; 0 falhas |
| Testes focados | Segurança, operações, busca e conversores aprovados em modo single-thread |
| Clippy host | Aprovado com `-D warnings` |
| Check/Clippy Windows GNU | Aprovados com `-D warnings` |
| Build release Windows GNU | Aprovado |
| `cargo audit` | Aprovado; warnings transitivos de manutenção já documentados |
| `cargo deny check` | Advisories, bans, licenças e fontes aprovados |
| Contrato de ativação | Aprovado |
| Contrato Windows nativo | `longPathAware`, fixture acima de MAX_PATH e junction aprovados |
| Contrato de nomes reservados | ASCII e sobrescritos COM/LPT aprovados |
| Layout Markdown | 85 arquivos, 0 violações e 0 links locais quebrados |
| Diff | `git diff --check` aprovado |

A suíte host não tenta fingir semântica case-insensitive ou atributos Windows. O cross-build comprova a compilação das ramificações Windows GNU; o teste `mklink /J` condicionado a Windows e os jobs nativos são os responsáveis pela prova comportamental do junction.

## Limitações honestas

A release não declara resolução de TOCTOU entre validação por caminho e uso, contenção de descendentes FFmpeg com Job Objects, UNC/SMB, namespaces extended-length, ACLs reais, arquivos bloqueados, volumes removíveis, disco cheio, associação inexistente, DPI por monitor, alto contraste, leitor de tela, drag-and-drop efetivo do Explorer ou restauração interativa da Lixeira. O risco do diretório atual na busca de backend permanece aberto como decisão de confiança e será tratado em ciclo próprio com fixture adversarial.

A distribuição continua portable, sem assinatura digital, instalador MSI/MSIX ou atualização automática. O binário release foi construído, mas a execução gráfica nativa Windows 10/11 não foi realizada nesta sessão; portanto a release preserva essa limitação na documentação.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-searchpathw "SearchPathW function"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa "CreateProcessA function"
[3]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points-and-file-operations "Reparse Points and File Operations"
[4]: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects "Job Objects"
