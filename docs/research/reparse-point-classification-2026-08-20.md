# Classificação de reparse points e junctions no Rovex

Data: 2026-08-20

## Risco investigado

O roadmap mantinha junctions e demais reparse points como risco pendente. A auditoria de `src/filesystem.rs` confirmou que a classificação de uma entrada usava somente `FileType::is_symlink()`, enquanto a própria documentação Windows define junctions como reparse points. A validação da raiz de `list_directory` verificava `symlink_metadata(path).is_dir()` e depois chamava `read_dir(path)`, sem uma rejeição explícita por atributo de reparse. Esse contrato permitia que uma junction fosse tratada como diretório navegável caso a biblioteca Rust a classificasse como diretório, deixando a decisão dependente de uma distinção de `FileType` que não cobre a categoria inteira de reparse points.

A documentação oficial afirma que junctions são implementadas por reparse points e que aplicações que usam `CreateFile` devem considerar `FILE_FLAG_OPEN_REPARSE_POINT` quando abrem um reparse point. Ela também destaca que mounted folders e linked files alteram o comportamento normal do filesystem. O Rovex adota a política conservadora de bloquear a navegação da raiz quando `FILE_ATTRIBUTE_REPARSE_POINT` está presente e classificar qualquer entrada reparse como `EntryKind::Symlink`, evitando seguir destinos externos.[1] [2] [3]

## Correção

`DirectoryEntry::from_path` agora testa `FILE_ATTRIBUTE_REPARSE_POINT` no Windows antes de `is_dir()` e `is_file()`. A raiz de `list_directory` faz a mesma verificação antes de chamar `read_dir`. Em Unix, o helper preserva a semântica anterior baseada em symlink.

A mudança não tenta resolver o destino, canonicalizar ou permitir junctions de forma seletiva. Ela evita que a UI navegue automaticamente por qualquer reparse point até que exista um contrato específico para cada tag.

## Regressões e validação

| Validação | Cobertura |
|---|---|
| Unitário Unix | Raiz symlinkada é recusada sem seguir o destino |
| Windows nativo | Fixture `New-Item -ItemType Junction` aponta para marcador externo; CLI precisa retornar erro |
| Gate estrutural | Exige fixture de junction, checagem de retorno zero e chamada no CI |
| Host | Testes de filesystem, conversores e Clippy |
| Cross-Windows GNU | Check, Clippy e build release |
| CI | Smoke nativo executado em `windows-latest` |

## Correção adicional descoberta

Durante o primeiro ciclo de validação, o teste `ffmpeg_fake_is_killed_when_cancelled` falhou uma vez em 103 testes. A repetição isolada passou, indicando uma corrida de timing na fixture: o teste cancelava após 100 ms sem confirmar que o fake backend já havia iniciado. O teste agora usa um arquivo marcador de readiness antes do cancelamento e inclui a variante de erro no diagnóstico. Cinco execuções focadas e três suítes completas passaram após a correção.

## Limitações

A fixture cobre junction local controlada, não todas as tags de reparse, mounted folders, OneDrive placeholders, DFS, UNC ou filtros de filesystem. Nenhum desses casos é declarado resolvido sem fixture nativa específica.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions "Hard Links and Junctions"
[2]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points-and-file-operations "Reparse Points and File Operations"
[3]: https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-point-tags "Reparse Point Tags"
