# Rovex v0.1.21 — bloqueio de reparse points e estabilidade dos testes

Data: 2026-08-20

A v0.1.21 corrige uma falha real de classificação no filesystem Windows e uma corrida de timing confirmada na suíte de processos. O risco de junction foi investigado comparando o código com a documentação oficial do Windows: junctions são reparse points, mas a listagem anterior só verificava `FileType::is_symlink()` e `is_dir()` antes de chamar `read_dir`. O ciclo adiciona rejeição explícita por `FILE_ATTRIBUTE_REPARSE_POINT`, regressão Unix, fixture de junction no Windows nativo e um gate estrutural para preservar a cobertura.

| Item | Resultado |
|---|---|
| Versão | `0.1.21` |
| Branch de backup | `backup/before-reparse-point-classification-20260820` |
| Filesystem | `src/filesystem.rs` |
| Smoke nativo | `scripts/verify_windows_native.ps1` |
| Gate estrutural | `scripts/test_windows_native_contract.sh` |
| Pesquisa | `docs/research/reparse-point-classification-2026-08-20.md` |

## Falha confirmada

A auditoria encontrou na raiz de `FileSystem::list_directory` esta sequência: `symlink_metadata(path)`, teste de `metadata.is_dir()` e depois `read_dir(path)`. A classificação de cada entrada também priorizava `FileType::is_symlink()` e só depois `is_dir()`.

A documentação oficial define junctions como diretórios implementados por reparse points e alerta que reparse points podem implementar mounted folders, linked files e comportamentos de filtros. Portanto, usar somente o predicado de symlink não é um contrato completo para impedir redirecionamentos no Windows. A falha foi tratada como confirmada no nível do contrato de código; a execução nativa agora fornece a prova comportamental por fixture.[1] [2] [3]

> A política do Rovex é conservadora: qualquer reparse point é não navegável até que exista um contrato específico para sua tag. O aplicativo não tenta descobrir ou seguir o alvo automaticamente.

## Correção

Foi adicionado um helper `is_reparse_point` que, no Windows, verifica `FILE_ATTRIBUTE_REPARSE_POINT` e, em Unix, preserva a detecção de symlink. `DirectoryEntry::from_path` agora classifica entradas reparse como `EntryKind::Symlink` antes de testar diretório ou arquivo. `list_directory` recusa uma raiz reparse antes de `read_dir`, retornando erro controlado com a razão `diretório redirecionado por link ou reparse point`.

Essa alteração evita navegação acidental por junctions, mounted folders, links NTFS e outras tags. Ela não altera a política de exclusão ou ativações para permitir qualquer reparse point.

Na primeira execução do CI, o smoke imprimiu que a junction havia sido rejeitada, mas o job terminou com código 1 porque o `LASTEXITCODE` não-zero esperado do `cargo run` permaneceu como último estado do PowerShell. Isso foi uma falha operacional confirmada do teste, não do bloqueio de segurança. O script agora copia o valor para `junctionExitCode`, valida o erro e zera `$global:LASTEXITCODE` antes de concluir; o gate estrutural exige esse tratamento para impedir regressão.

## Regressão adicional descoberta

Na primeira validação após a mudança, o teste `ffmpeg_fake_is_killed_when_cancelled` falhou uma vez entre 103 testes. A repetição isolada e três suítes completas passaram, confirmando uma corrida de timing na fixture: o teste cancelava após 100 ms sem saber se o fake backend já havia iniciado o processo de longa duração.

O teste agora cria um marcador de readiness no próprio fake backend, espera até dois segundos por esse marcador e só então aciona o cancelamento. O `assert` também inclui o erro recebido em caso de falha. Cinco execuções focadas e uma suíte completa passaram após a correção, com 104 testes aprovados.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| Testes host | 104 aprovados; 2 ignorados explicitamente |
| Regressão Unix | Raiz symlinkada recusada sem seguir o destino |
| Smoke Windows | Junction real com marcador externo deve retornar erro e o script deve concluir com código zero |
| Long path | Smoke existente acima de 260 caracteres preservado |
| Gate estrutural | Junction, retorno não-zero, manifesto e CI verificados |
| Check/Clippy host | Aprovados sem warnings |
| Check/Clippy/build Windows GNU | Aprovados antes do CI final |
| Auditoria documental | Links e layout aprovados |

## Limitações honestas

A fixture cobre uma junction local criada no runner, mas não todas as tags de reparse. Mounted folders, OneDrive placeholders, DFS, UNC, extended-length, filtros de filesystem, ACLs e arquivos bloqueados continuam exigindo fixtures nativas específicas. O bloqueio amplo é intencional e não significa que todas essas integrações foram testadas.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions "Hard Links and Junctions"
[2]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points-and-file-operations "Reparse Points and File Operations"
[3]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-point-tags "Reparse Point Tags"
