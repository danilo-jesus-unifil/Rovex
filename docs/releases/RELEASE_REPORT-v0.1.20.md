# Rovex v0.1.20 — validação nativa de caminhos longos

Data: 2026-08-20

A v0.1.20 incrementa a validação Windows do Rovex. A auditoria não encontrou uma falha funcional já comprovada no suporte a caminhos longos; encontrou uma lacuna verificável no pipeline: o manifesto já declarava `longPathAware`, mas o smoke Windows nativo não criava nem listava uma árvore acima de 260 caracteres. O lote corrige essa ausência de evidência sem declarar suporte a namespaces que ainda não foram testados.

| Item | Resultado |
|---|---|
| Versão | `0.1.20` |
| Branch de backup | `backup/before-native-longpath-smoke-20260820` |
| Smoke nativo | `scripts/verify_windows_native.ps1` |
| Gate estrutural | `scripts/test_windows_native_contract.sh` |
| Manifesto | `longPathAware=true` exigido pelo verificador |
| Pesquisa | `docs/research/long-path-validation-2026-08-20.md` |

## Investigação

O issue #2 e o roadmap listavam caminhos longos e UNC como riscos pendentes. A inspeção confirmou que `assets/rovex.manifest` já contém o elemento `longPathAware` no namespace Windows 10/11 e que `scripts/verify_windows_manifest.sh` já o exigia no PE. Entretanto, `scripts/verify_windows_native.ps1` testava somente uma pasta temporária curta, Unicode, espaços e listagem comum.

Essa é uma lacuna real de validação: o projeto podia afirmar que o manifesto tinha a declaração, mas o CI não demonstrava que o binário conseguia atravessar uma árvore de filesystem acima de MAX_PATH. Não foi correto tratar isso como falha funcional antes de uma execução nativa.

A documentação oficial explica que Windows 10 versão 1607 ou posterior remove MAX_PATH de muitas APIs somente quando a configuração do sistema está habilitada e o aplicativo declara `longPathAware`. Ela também diferencia UNC normal (`\\server\\share`) de caminhos extended-length (`\\?\\C:\\...` e `\\?\\UNC\\server\\share`) e alerta que o Shell e o filesystem podem ter requisitos diferentes.[1] [2]

## Correção implementada

O smoke Windows agora cria quatro níveis com componentes longos dentro do fixture do runner, grava um arquivo cujo caminho excede 260 caracteres, falha se o caminho não ultrapassar esse limite e executa `cargo run --quiet -- --cli` sobre o diretório longo. A saída precisa manter o nome do arquivo e o código de saída precisa ser zero. O cleanup continua limitado ao fixture temporário.

O novo `test_windows_native_contract.sh` verifica que o manifesto contém `longPathAware`, que o script mede explicitamente `> MAX_PATH`, que o CLI é executado com o diretório longo e que o workflow chama o smoke no job Windows nativo. Esse gate roda na matriz de qualidade e evita regressões documentais ou remoção silenciosa do único teste real.

Nenhuma conversão automática para `\\?\\` foi adicionada. Isso evita mascarar diferenças entre filesystem e Shell e evita declarar UNC ou extended-length sem um compartilhamento controlado.

## Evidências e critérios

| Verificação | Resultado esperado |
|---|---|
| Manifesto embutido | `longPathAware` presente |
| Fixture de caminho | Comprimento efetivo maior que 260 |
| Listagem nativa | CLI retorna código zero e preserva o nome do arquivo |
| Gate estrutural | Manifesto, fixture e workflow presentes |
| UNC normal | Pendente de compartilhamento Windows controlado |
| `\\?\\` extended-length | Pendente de fixture Windows controlada |
| Junction/mounted folder | Pendente; não simulado por strings |
| ACL/arquivo bloqueado | Pendente de execução nativa específica |

## Limitações honestas

O smoke depende de `LongPathsEnabled` e das políticas do runner Windows, além do manifesto. A passagem no runner prova a árvore local criada pelo teste, mas não prova todos os volumes, compartilhamentos de rede, junctions, mounted folders ou associações do Shell. UNC, extended-length, ACLs e arquivos bloqueados continuam documentados como trabalho futuro e não foram marcados como resolvidos.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation "Maximum Path Length Limitation"
[2]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file "Naming a File"
[3]: https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats "File path formats on Windows systems"
