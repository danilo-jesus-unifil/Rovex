# CI nativo Windows: pesquisa 2026-08-19

## Decisões

O GitHub documenta `windows-latest` como imagem estável atual de runner hospedado e distingue essa imagem de runners self-hosted, que exigem administração própria [1] [2]. O workflow do Rovex deve usar o runner hospedado para uma verificação nativa repetível, sem depender de uma máquina pessoal ou de credenciais do usuário.

A sintaxe oficial do GitHub Actions permite limitar permissões no nível do workflow; `contents: read` basta para checkout e inspeção do repositório, enquanto publicar releases exigiria permissões maiores e não será colocado no job de verificação [3]. O job não deve criar tags, publicar releases, alterar issues ou usar secrets.

`actions/upload-artifact` é apropriado para compartilhar artefatos de uma execução, mas o objetivo deste lote é validar o PE e o CLI; não é necessário fazer upload do EXE em cada push. O pacote portable continuará sendo gerado e publicado explicitamente no pipeline de release, não em todo build de CI [4].

| Verificação | Runner | Permissão | Resultado esperado |
|---|---|---|---|
| `cargo fmt`, check, test, Clippy | `windows-latest` | `contents: read` | Sem erros no alvo nativo |
| CLI `--cli .` | `windows-latest` | Nenhuma além do checkout | Lista o diretório real sem abrir UI |
| Release PE | `windows-latest` | `contents: read` | `target/release/rovex.exe` criado |
| Manifesto Windows | `windows-latest` | `contents: read` | Verificação PowerShell de bytes/PE limitada ao que o runner fornece |
| Smoke gráfico Xvfb | Linux | `contents: read` | Permanece no job Linux existente; Windows não recebe automação de mouse falsa |

A execução nativa do CLI comprova somente a entrada Win32 e a listagem básica; não comprova DPI, Explorer drag-and-drop, Lixeira real, ACLs, Shell context menu ou acessibilidade. Essas lacunas permanecem explícitas na matriz de compatibilidade.

## Referências

[1]: https://docs.github.com/actions/using-github-hosted-runners/about-github-hosted-runners "GitHub-hosted runners: GitHub Docs"

[2]: https://docs.github.com/en/actions/reference/runners/github-hosted-runners "GitHub-hosted runners reference: GitHub Docs"

[3]: https://docs.github.com/actions/using-workflows/workflow-syntax-for-github-actions "Workflow syntax for GitHub Actions: GitHub Docs"

[4]: https://docs.github.com/en/actions/tutorials/store-and-share-data "Store and share data with workflow artifacts: GitHub Docs"
