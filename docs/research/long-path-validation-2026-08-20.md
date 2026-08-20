# Validação de caminhos longos e UNC no Windows

Data: 2026-08-20

## Falha candidata e resultado da investigação

O roadmap do issue #2 listava caminhos longos e UNC como riscos pendentes. A auditoria confirmou uma **lacuna de validação**, não uma falha funcional já comprovada: o manifesto `assets/rovex.manifest` já declara `longPathAware=true`, mas `scripts/verify_windows_native.ps1` testava apenas Unicode, espaços e listagem comum. Nenhum smoke nativo criava uma árvore acima de 260 caracteres.

A referência oficial informa que Windows 10 versão 1607 ou posterior exige tanto a configuração do sistema `LongPathsEnabled` quanto o elemento `longPathAware` no manifesto para remover a limitação MAX_PATH em muitas APIs Win32. Também diferencia caminhos UNC (`\\server\\share`) e caminhos extended-length (`\\?\\C:\\...` e `\\?\\UNC\\server\\share`). O Shell e o filesystem podem ter requisitos diferentes; portanto, um cross-build não é evidência suficiente.

## Correção deste ciclo

O smoke Windows nativo agora cria quatro diretórios com componentes longos e um arquivo cujo caminho excede 260 caracteres. Ele executa o CLI real com essa pasta, exige código de saída zero e verifica que o nome do arquivo aparece na listagem. Se a árvore não ultrapassar 260 caracteres, o teste falha explicitamente. A execução usa o mesmo binário compilado pelo fluxo Windows e o mesmo manifesto que será distribuído.

Não foi adicionada normalização cega de caminhos nem conversão automática para `\\?\\`. O resultado dessa validação deve orientar uma futura mudança apenas se um runner Windows reproduzir uma falha real. UNC e extended-length continuam pendentes porque exigem um compartilhamento controlado e não devem ser simulados com strings sem filesystem real.

## Gates incrementados

| Gate | Cobertura |
|---|---|
| `verify_windows_native.ps1` | Unicode, espaços, diretório real e caminho com mais de 260 caracteres |
| Manifesto | `longPathAware` continua exigido por `verify_windows_manifest.sh` |
| CI | O smoke ampliado roda no `windows-latest` em cada push/PR |
| Limite honesto | UNC/extended-length só serão declarados após fixture nativa controlada |

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation "Maximum Path Length Limitation"
[2]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file "Naming a File"
[3]: https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats "File path formats on Windows systems"
