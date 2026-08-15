# Auditoria inicial — conversores, toolbar e locais — 2026-08-15

## Prompt lido

O PROMPTMASTER foi lido integralmente. As prioridades relevantes para esta rodada são segurança, correção, estabilidade, compatibilidade com Windows 10, desempenho, UI não bloqueante, ausência de simulação, resolução de erros, regressões cobertas por testes e validação do arquivo convertido antes de informar sucesso.

## Inconsistências e riscos identificados

| Área | Estado observado | Risco ou causa provável | Regra de correção |
|---|---|---|---|
| Backend de conversão | `src/converters.rs` usa diretamente `Command::new("ffmpeg")` e `Command::new("ffprobe")`. | Um processo desktop iniciado pelo Explorer pode não enxergar o PATH atualizado do usuário; o usuário pode possuir executáveis com nomes diferentes, em diretórios de instalação comuns, ou `ffmpeg.exe` pode estar ao lado do executável. | Resolver executável explicitamente com tentativas determinísticas e seguras; testar PATH herdado, PATH do usuário/sistema, diretório do Rovex, overrides absolutos e diretórios fixos de instalação conhecidos; não usar o diretório de trabalho como fallback implícito, pois ele pode ser controlado pelo conteúdo aberto pelo usuário. Nunca usar shell, executar arquivo não regular ou procurar recursivamente em diretórios arbitrários. |
| Diagnóstico de PATH | `BackendUnavailable` só informa “não foi encontrado no PATH”. | A mensagem não revela quais estratégias foram tentadas nem oferece diagnóstico suficiente. | Incluir no erro o backend lógico, tentativas resumidas e instrução clara para instalar FFmpeg/ffprobe no PATH; preservar o erro real e nunca declarar conversão concluída. |
| Validação de backend | A implementação inicia FFmpeg e só detecta ausência no momento de `spawn`. | Falhas do PATH são descobertas tarde, uma vez por conversão, e não há teste unitário do resolvedor. | Resolver e validar o executável antes do worker; adicionar testes com PATH controlado, caminho explícito e fallback; manter o worker isolado. |
| Codec | A seleção por extensão é intencionalmente limitada, mas extensão não prova conteúdo. | Um arquivo renomeado pode alcançar o backend e falhar de forma pouco clara. | Manter validação por ffprobe da saída e melhorar diagnóstico de entrada; não habilitar diretórios ou extensões incompatíveis. |
| Toolbar | Os três botões de navegação não têm largura explícita; dependem do conteúdo “←”, “→” e “↑”. O layout usa gap global, mas os controles parecem comprimidos. | A largura natural mínima e a distribuição ao lado do caminho tornam os alvos pequenos e visualmente grudados. | Definir largura mínima explícita, espaçamento horizontal dedicado e `horizontal-stretch` apenas no caminho; separar o botão Atualizar por gap e padding visual. |
| Locais padrão | `default_locations` usa apenas `HOME`/`USERPROFILE` e concatena Desktop, Documents e Downloads. | No Windows, pastas conhecidas podem ser redirecionadas, localizadas, removidas ou movidas para OneDrive; `home.join` não representa Known Folders. | Usar `SHGetKnownFolderPath` com KNOWNFOLDERID no Windows, aceitar apenas diretórios existentes, remover duplicatas e manter fallback compatível. Em Linux, preservar HOME e diretórios convencionais existentes. |
| Dependência Windows | O projeto não usa diretamente a API Known Folders. | Adicionar dependência indiscriminadamente aumentaria a árvore e poderia quebrar o alvo Linux. | Adicionar `windows-sys` somente em `cfg(windows)`, com features mínimas `Win32_Foundation`, `Win32_System_Com` e `Win32_UI_Shell`; validar lockfile, licença, audit e cross-build. |

## Pesquisa oficial

A documentação da Microsoft afirma que, para código novo, Known Folders devem ser obtidas por `SHGetKnownFolderPath` com constantes `KNOWNFOLDERID`, pois essas pastas podem ser redirecionadas, inclusive para locais de rede. A lista oficial inclui `FOLDERID_Desktop`, `FOLDERID_Documents`, `FOLDERID_Downloads`, `FOLDERID_Music`, `FOLDERID_Objects3D`, `FOLDERID_Pictures` e `FOLDERID_Videos` [1] [2].

A documentação atual de `std::process::Command` informa que, no Windows, o Rust resolve o executável antes de chamar o processo e pesquisa o PATH do filho explicitamente configurado, o diretório do executável atual, diretórios do sistema, diretório do Windows e o PATH do processo pai. Também confirma que argumentos separados não passam por shell [3]. Isso torna correto manter `Command::arg`/`args`, mas não suficiente para uma experiência robusta quando o PATH herdado pelo processo desktop está desatualizado ou quando o backend foi instalado em um local comum que não está no PATH.

## Plano técnico desta rodada

A correção implementa um resolvedor de backend com ordem de tentativas determinística, validação de arquivo regular e execução por caminho absoluto. As tentativas são deduplicadas e cada backend (`ffmpeg` e `ffprobe`) é resolvido separadamente, sem confiar que ambos estejam na mesma pasta. A cadeia considera override absoluto, PATH herdado, diretório do Rovex, diretórios de usuário usados por gerenciadores conhecidos e diretórios fixos de instalação; não faz busca recursiva nem usa o diretório de trabalho implícito. O conversor só inicia após os dois executáveis serem resolvidos, e os erros conservam timeout, cancelamento, validação por ffprobe, publicação sem sobrescrita e remoção de temporários.

A toolbar receberá largura mínima e gap explícitos, sem reduzir o espaço do caminho de forma imprevisível. Os locais padrão receberão Known Folders reais no Windows e fallback por ambiente no Linux, sempre filtrados por existência e sem presumir que todas as pastas existam.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/shell/known-folders "Microsoft Learn — Known Folders"
[2]: https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid "Microsoft Learn — KNOWNFOLDERID"
[3]: https://doc.rust-lang.org/std/process/struct.Command.html "Rust standard library — std::process::Command"

## Verificação intermediária

A captura release `artifacts/rovex-dark-theme.png` mostrou os três botões de navegação com largura confortável e gaps visíveis, o botão Atualizar separado do filtro e a interface escura preservada. No Linux do ambiente, apenas os locais existentes foram exibidos; no Windows, a fonte passa a ser a API Known Folders.

O teste real dos quatro conversores passou duas vezes: primeiro com `ROVEX_FFMPEG_PATH=/usr/bin/ffmpeg` e `ROVEX_FFPROBE_PATH=/usr/bin/ffprobe`, depois com ambos os overrides apontando para caminhos inexistentes. A segunda execução comprovou que o resolvedor ignora overrides ausentes e encontra os binários no PATH/locais seguros seguintes.
