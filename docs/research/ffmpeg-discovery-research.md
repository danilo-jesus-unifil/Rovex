# Pesquisa de descoberta de FFmpeg/ffprobe

## Evidências oficiais iniciais — 2026-08-17

### Rust `std::process::Command`

Fonte: https://doc.rust-lang.org/std/process/struct.Command.html

A documentação oficial informa que, no Windows, o Rust resolve o executável antes de criar o processo. Para um nome não absoluto, a ordem descrita inclui: PATH do processo filho quando explicitamente definido, diretório do executável atual, diretório do sistema, diretório do Windows e PATH do processo pai. A própria documentação recomenda caminho absoluto ou PATH explicitamente controlado para evitar surpresas.

Implicação para o Rovex: `Command::new("ffmpeg")` não é uma descoberta confiável quando o aplicativo gráfico herdou um PATH antigo ou diferente do terminal do usuário. A solução deve localizar e validar um caminho absoluto antes do spawn, preservando argumentos estruturados.

### Win32 `CreateProcessA`

Fonte: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa

A documentação do Windows descreve que, com nome simples e caminho relativo, a busca passa pelo diretório de carregamento da aplicação, diretório atual do processo pai, diretórios do sistema, diretório do Windows e, por fim, diretórios do PATH. Um caminho absoluto ignora essa busca e especifica exatamente o executável. A documentação também afirma que a busca nativa não consulta a chave de Registro `App Paths`; para incluí-la, o Windows recomenda `ShellExecute`.

Implicação para o Rovex: usar caminho absoluto após uma resolução própria é preferível. `App Paths` pode ser uma camada opcional de descoberta, mas não deve ser invocada por shell nem confundida com execução arbitrária. O resolvedor deve validar arquivo regular, arquitetura/execução e identidade do backend antes de aceitar o candidato.

## Relatos de usuários

### Audacity no Windows

Fonte: https://forum.audacityteam.org/t/error-message-ffmpeg-not-found-in-your-path-solved/65525

Um usuário relatou que o FFmpeg aparecia como instalado/reconhecido nas preferências do Audacity, mas a exportação continuava informando “ffmpeg not found in your path” depois de reinstalar o aplicativo. O caso é relevante porque mostra que reconhecimento/configuração interna não garante que o processo de conversão consiga resolver o executável no momento do uso; trata-se de um aplicativo gráfico, não apenas de um terminal.

Implicação: o Rovex não deve confiar apenas em uma verificação inicial nem no PATH herdado. Ele precisa resolver novamente no momento do clique e usar caminho absoluto no worker, mantendo o diagnóstico da tentativa de spawn.

### Stack Overflow — caminho absoluto e reinício do processo

Fonte: https://stackoverflow.com/questions/66727950/ffmpeg-not-found-relativ-path-install-ffmpeg

O relato descreve que uma configuração relativa falhava, enquanto o caminho absoluto funcionava. A resposta recomendou adicionar a pasta que contém `ffmpeg.exe` ao PATH e reiniciar o cmd/PowerShell para que o novo PATH fosse herdado. O autor também relatou que queria distribuir o programa em outra máquina sem exigir PATH pré-configurado e que o caminho absoluto era a alternativa que funcionava; a discussão menciona acrescentar a pasta ao PATH do processo.

Implicação: há duas soluções distintas que não devem ser confundidas: atualizar/reiniciar o ambiente do processo e resolver um caminho absoluto próprio. Para um aplicativo distribuído, o fallback mais confiável é localizar o executável e passar o caminho absoluto, sem exigir que o usuário reinicie o Explorer ou abra outro terminal.

## Mais relatos sobre PATH desatualizado e instalação

### Super User — Windows 10/11

Fonte: https://superuser.com/questions/1716400/ffmpeg-is-not-recognized-as-an-internal-or-external-command-operal-program-or

A solução aceita recomenda adicionar ao PATH a pasta que contém `ffmpeg.exe`, não o arquivo individual, usando o separador `;` do Windows. O relato também registra que, em um caso do Windows 11, reiniciar o computador resolveu porque atualizou as variáveis de PATH disponíveis aos processos.

Implicação: pedir ao usuário para reiniciar o Explorer, terminal ou computador é uma solução operacional, não uma boa recuperação automática do Rovex. O programa deve reconsultar o ambiente persistente e resolver um caminho absoluto no momento da conversão.

### GitHub — projeto jumpcutter

Fonte: https://github.com/carykh/jumpcutter/issues/65

Usuários relataram o erro “ffmpeg is not recognized as an internal or external command” mesmo após instalar o FFmpeg. As respostas destacam que é necessário adicionar ao PATH a pasta que contém `FFMPEG.EXE` e reabrir o Command Prompt; também aparece a recomendação de confirmar que o executável foi realmente colocado na pasta esperada.

Implicação: “instalado” e “encontrado pelo processo atual” são estados diferentes. O fallback precisa diferenciar: (a) arquivo instalado, mas fora do PATH do processo; (b) PATH persistente alterado depois do processo iniciar; (c) arquivo ausente; e (d) candidato encontrado, mas incapaz de executar.

## App Paths e SearchPathW — documentação oficial

### App Paths

Fonte: https://learn.microsoft.com/en-us/windows/win32/shell/app-registration

A Microsoft recomenda registrar aplicações em `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths` para instalações por usuário e em `HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths` para instalações por máquina. O subkey deve ter o nome do executável; o valor padrão `(Default)` contém o caminho totalmente qualificado e o valor `Path`, opcional, fornece diretórios adicionais que o Shell prefixa ao PATH do processo iniciado via `ShellExecuteEx`. Os valores podem ser `REG_SZ`, e `Path` também pode ser `REG_EXPAND_SZ` em Windows 7+.

Correção importante: a documentação do `CreateProcess` afirma que ele não consulta App Paths, enquanto a documentação de App Paths atribui esse mecanismo ao Shell. Portanto, o Rovex deve ler App Paths diretamente como uma camada de descoberta read-only; não deve usar `ShellExecute`/`ShellExecuteEx` para iniciar conversões, pois isso poderia introduzir verbos, associação de arquivos, quoting implícito e comportamento de shell que conflitam com o requisito de segurança. A entrada descoberta deve ser validada e depois executada por `Command` com caminho absoluto e argumentos separados.

### SearchPathW

Fonte: https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-searchpathw

`SearchPathW` procura um arquivo em um caminho especificado. Quando `lpPath` é `NULL`, usa uma ordem de busca dependente da configuração do sistema e do modo `SafeProcessSearchMode`; a documentação descreve a interação com o PATH e o diretório de trabalho. O retorno indica o tamanho da string encontrada ou o tamanho necessário quando o buffer é pequeno, permitindo redimensionar com segurança. Windows 10 1607+ oferece suporte à remoção do limite `MAX_PATH` quando o processo está preparado para caminhos longos.

Implicação: `SearchPathW` é útil como camada de resolução nativa do PATH, mas não deve ser tratado como garantia de consulta a App Paths. Deve ser chamado com buffer redimensionável, verificar o retorno e validar o arquivo resultante. Como a ordem com `lpPath = NULL` pode incluir diretório atual e sofrer ambiguidade, a resolução deve ocorrer com nomes controlados (`ffmpeg.exe`/`ffprobe.exe`) e os candidatos precisam ser filtrados antes de executar.

## WinGet, Chocolatey e Scoop

### WinGet — pacote portátil e Links

Fontes: https://github.com/microsoft/winget-cli/issues/3601 e https://github.com/microsoft/winget-cli/issues/5557

Issues do repositório oficial do WinGet mostram que pacotes CLI podem adicionar caminhos longos sob `%LOCALAPPDATA%\\Microsoft\\WinGet\\Packages\\<package>...` ao PATH, e que o WinGet usa diretórios de links para expor executáveis portáteis. O issue #5557 registra especificamente `%LOCALAPPDATA%\\Microsoft\\WinGet\\Links` como diretório de Links do usuário; versões e configurações também podem usar `C:\\Program Files\\WinGet\\Links` para o escopo da máquina. O issue #3601 descreve problemas de PATH excessivamente longo e observa que Chocolatey e Scoop usam shims em diretórios curtos comuns.

Implicação: o Rovex deve procurar primeiro nos diretórios de Links (`%LOCALAPPDATA%\\Microsoft\\WinGet\\Links` e, quando existente, `%ProgramFiles%\\WinGet\\Links`) e depois, com profundidade limitada, em diretórios de pacote WinGet que contenham `ffmpeg.exe` ou `ffprobe.exe`. A busca deve enumerar apenas diretórios previsíveis, não fazer varredura recursiva do disco, respeitar limites de quantidade/profundidade e validar cada arquivo antes de aceitar. O PATH não deve ser considerado confiável quando o WinGet acabou de instalar ou atualizar um pacote.

### Chocolatey e Scoop — shims

A mesma documentação comunitária registrada no issue do WinGet indica que Chocolatey normalmente expõe shims em `C:\\ProgramData\\chocolatey\\bin` e Scoop em `%USERPROFILE%\\scoop\\shims`, enquanto os binários reais podem ficar em subdiretórios de instalação. Isso justifica procurar os shims e também os diretórios `current\\bin`/`tools\\...\\bin` conhecidos, mas sempre validar o alvo final e tratar links/atalhos conforme a política de segurança do Rovex.

## Relato de aplicação gráfica que exigia caminho absoluto

### MediaMTX — Windows

Fonte: https://github.com/bluenviron/mediamtx/issues/3582

O issue descreve que, no Windows, um comando de transcodificação não conseguia executar `ffmpeg` por caminho relativo mesmo com variáveis de ambiente configuradas. O autor encerrou o caso após descobrir que havia colocado o aplicativo e o FFmpeg no mesmo diretório, o que interferia na descoberta/execução; a formulação do erro também registrava que o caminho absoluto era necessário.

Implicação: além do resolvedor, o Rovex deve evitar depender do diretório de trabalho atual ou de uma relação implícita entre o executável do aplicativo e o backend. Cada tentativa deve carregar um `PathBuf` absoluto já validado, e o worker deve configurar apenas o diretório de trabalho necessário para os arquivos de entrada/saída, nunca para “ajudar” a resolução do executável.

## `where.exe` e PowerShell `Get-Command`

### `where.exe`

Fonte: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/where

O comando oficial `where` mostra a localização de arquivos que correspondem ao padrão. Por padrão, procura no diretório atual e nos diretórios do PATH; também oferece `/q` para obter apenas o código de saída e `/f` para imprimir resultados entre aspas. A documentação confirma que o comando não executa o arquivo encontrado.

Implicação: `where.exe` é um fallback de diagnóstico/descoberta razoável, mas não é uma resolução independente de PATH nem uma consulta garantida a App Paths. O Rovex deve invocá-lo por caminho absoluto (`%SystemRoot%\\System32\\where.exe` quando validado) ou por um candidato confiável, usar `Command::arg("ffmpeg.exe")`, limitar stdout, aceitar apenas linhas que sejam caminhos absolutos existentes e regulares e, depois, executar somente o caminho validado. Não usar `/r` em discos inteiros: isso seria lento e invasivo.

### PowerShell `Get-Command`

Fonte: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/get-command?view=powershell-7.6

A documentação do PowerShell descreve `Get-Command` como um cmdlet que obtém comandos disponíveis na sessão e inclui executáveis nativos encontrados nos diretórios do `$Env:PATH`. Ele pode retornar metadados de um comando sem executar o executável consultado.

Implicação: `Get-Command` pode ser um fallback opcional de último recurso, porém acrescenta dependência de `powershell.exe`/`pwsh.exe`, parsing de saída e uma superfície maior que `where.exe`. Deve ser desabilitado se PowerShell não estiver em um caminho conhecido, invocado com argumentos separados e modo não interativo, sem perfil (`-NoProfile`), sem comando de rede e sem avaliar entrada do usuário. Na ordem do Rovex, `Get-Command` é menos prioritário que resolução direta, registro, SearchPathW, `where.exe` e diretórios conhecidos.

## Padrão de um projeto Rust especializado

### `ffmpeg-sidecar`

Fontes: https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/ e https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/paths/index.html

A documentação do crate Rust `ffmpeg-sidecar` expõe um módulo `paths` específico para localizar binários. A função `ffmpeg_path` recomenda primeiro procurar um FFmpeg adjacente ao executável Rust e, se falhar, invocar `ffmpeg` esperando que esteja no PATH. O crate também oferece utilitários separados para o caminho esperado do sidecar e para download/descompactação de binários.

Implicação: o padrão “adjacente ao aplicativo → PATH” é validado por um projeto Rust real e é compatível com distribuição controlada, mas é insuficiente para o Rovex porque não cobre PATH persistente do Registro, App Paths, WinGet Links, `SearchPathW`, shims e `ffprobe`. O Rovex deve aproveitar a ideia de retornar um caminho absoluto para `Command::new`, mas manter a política explícita de não baixar executáveis em runtime.

## Aplicativos gráficos similares e segurança

### HandBrake

Fonte: https://github.com/HandBrake/HandBrake/pull/6548

Em uma mudança recente do HandBrake, os mantenedores observaram que, em sistemas sem NVIDIA, a busca de DLLs poderia atravessar várias pastas do PATH do usuário, inclusive pastas graváveis pelo usuário. Mesmo considerando o risco baixo no contexto do aplicativo, a correção endureceu o carregamento para usar apenas `System32`, confirmada por ProcMon e testada no Windows 10+ via MinGW.

Implicação: localizar FFmpeg não deve significar aceitar qualquer candidato ou ampliar indiscriminadamente o PATH. O Rovex deve priorizar caminhos absolutos explícitos, diretório próprio do aplicativo e fontes de instalação conhecidas; registrar a origem do candidato; rejeitar diretórios não confiáveis quando possível; e não modificar o PATH global. Para DLLs carregadas pelo FFmpeg, a execução deve ocorrer de forma controlada e a validação do executável deve preceder a conversão.

### VLC

Fonte: https://wiki.videolan.org/VLC_command-line_help/

A documentação do VLC mostra que, em instalação padrão no Windows, os arquivos auxiliares e o relatório de ajuda são tratados em diretórios previsíveis da instalação, como `%PROGRAMFILES%\\VideoLAN\\VLC`. Isso não é uma implementação de busca de FFmpeg externo, mas ilustra a estratégia de aplicativos gráficos maduros: distribuir e consultar recursos por caminhos relativos ao diretório conhecido da aplicação, em vez de presumir um PATH global.

Implicação: o Rovex deve manter “FFmpeg ao lado do Rovex” e diretórios de instalação conhecidos como camadas fortes, antes de mecanismos de shell. A resolução deve ser observável e registrar qual estratégia encontrou o backend.

## Síntese técnica e plano de fallback

A pesquisa mostra que o erro não tem uma causa única. Em aplicativos gráficos Windows, o processo pode ter um PATH herdado antes da instalação do FFmpeg; o executável pode estar em uma pasta conhecida, mas fora do PATH; o gerenciador pode expor um shim em `Links`, `bin` ou `shims`; o usuário pode ter uma entrada App Paths válida; ou a resolução pode encontrar um candidato que não é executável. Por isso, a solução deve combinar descoberta determinística, fontes nativas do Windows, recuperação limitada e validação real.

| Ordem | Estratégia | Implementação planejada | Política de segurança e sucesso |
|---:|---|---|---|
| 1 | Override absoluto | `ROVEX_FFMPEG_PATH` e `ROVEX_FFPROBE_PATH` | Aceitar somente caminho absoluto, arquivo regular ou link/junction para arquivo regular; não aceitar diretório; validar existência e execução posterior. |
| 2 | PATH herdado | Reutilizar `Command`/`SearchPathW` com o ambiente atual | Resolver no clique, não apenas na inicialização; guardar caminho absoluto; não montar shell command. |
| 3 | PATH persistente | Ler PATH de HKCU/HKLM, expandir `REG_EXPAND_SZ`, deduplicar e procurar diretamente | Não alterar Registro nem PATH global; limitar comprimento, normalizar separadores e validar candidatos. |
| 4 | App Paths por usuário | Ler `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\ffmpeg.exe` e `ffprobe.exe` | Usar somente o valor `(Default)` como caminho candidato e opcionalmente `Path` para contexto; não usar `ShellExecute`. |
| 5 | App Paths por máquina | Ler a mesma chave em HKLM, contemplando a visão de Registro correspondente ao processo | Tratar ausência/permissão como falha não fatal; evitar confiar em valores que apontem para diretórios ou arquivos ausentes. |
| 6 | Busca nativa | Chamar `SearchPathW` com buffer redimensionável e nomes controlados | Usar como descoberta de PATH/sistema; verificar retorno, tamanho e arquivo resultante; não afirmar que consulta App Paths. |
| 7 | Links e shims | WinGet Links, Chocolatey `bin`, Scoop `shims` e `current\\bin` | Não fazer varredura recursiva ampla; aceitar somente arquivos candidatos conhecidos e registrar a origem. |
| 8 | Diretório do Rovex | Diretório do executável, subdiretórios controlados e diretório irmão | Priorizar distribuição lado a lado; caminho absoluto e validação antes do worker. |
| 9 | Diretórios conhecidos | WinGet Packages com profundidade limitada, Chocolatey, Scoop, `%ProgramFiles%`, `%LOCALAPPDATA%`, instalação manual | Enumerar apenas raízes previsíveis, limite de profundidade/quantidade e sem baixar artefatos. |
| 10 | `where.exe` | Executar `System32\\where.exe` com `ffmpeg.exe`/`ffprobe.exe` | Fallback de último recurso; parsear linhas, rejeitar stdout ambíguo, validar cada caminho; não usar `/r`. |
| 11 | PowerShell opcional | `powershell.exe -NoProfile -NonInteractive -Command ... Get-Command ...` | Somente se fontes anteriores falharem; sem perfil, rede, avaliação de entrada ou execução do FFmpeg; parsing estrito. |
| 12 | Diagnóstico final | Retornar erro com estratégias tentadas e caminhos rejeitados | Não simular conversão; não declarar sucesso sem arquivo de saída existente e verificável. |

A ordem acima corrige uma conclusão potencialmente perigosa: **`where.exe` e `SearchPathW` não devem ser tratados como consultas garantidas a App Paths**. `where` procura o diretório atual e PATH; `SearchPathW` segue a busca nativa configurável. App Paths deve entrar por leitura explícita do Registro, e a execução deve continuar sendo feita por `std::process::Command` com caminho absoluto e argumentos separados.

### Estratégia de tentativa da conversão

Cada conversão deve resolver `ffmpeg` e, quando necessária, `ffprobe` com o mesmo resolvedor, mas sem assumir que ambos estão na mesma pasta. Para cada par de candidatos, o worker deve executar a verificação real de versão ou uma operação de metadados controlada; se o spawn falhar, registrar a causa e seguir para o próximo candidato. Se o processo iniciar e retornar código diferente de zero, a tentativa deve ser marcada como falha, o próximo candidato pode ser tentado apenas quando a falha for de descoberta/execução do backend, e o resultado parcial deve ser removido com segurança. Ao final, a UI deve receber um erro explicando que nenhuma camada conseguiu produzir uma saída válida.

A conversão só será considerada concluída quando o processo terminar com sucesso, o arquivo de saída existir, for arquivo regular, tiver tamanho compatível com a operação e estiver acessível. A validação deve ocorrer fora do event loop, com cancelamento cooperativo e sem bloquear a UI.

## Referências

[1]: https://doc.rust-lang.org/std/process/struct.Command.html "Rust std::process::Command"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa "Microsoft CreateProcessA"
[3]: https://forum.audacityteam.org/t/error-message-ffmpeg-not-found-in-your-path-solved/65525 "Audacity Forum — FFMPEG not found in your path"
[4]: https://stackoverflow.com/questions/66727950/ffmpeg-not-found-relativ-path-install-ffmpeg "Stack Overflow — ffmpeg not found, relative path"
[5]: https://superuser.com/questions/1716400/ffmpeg-is-not-recognized-as-an-internal-or-external-command-operal-program-or "Super User — ffmpeg is not recognized"
[6]: https://github.com/carykh/jumpcutter/issues/65 "GitHub jumpcutter issue #65"
[7]: https://learn.microsoft.com/en-us/windows/win32/shell/app-registration "Microsoft Application Registration / App Paths"
[8]: https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-searchpathw "Microsoft SearchPathW"
[9]: https://github.com/microsoft/winget-cli/issues/3601 "WinGet issue #3601 — PATH length and shims"
[10]: https://github.com/microsoft/winget-cli/issues/5557 "WinGet issue #5557 — repeated prefixes and Links"
[11]: https://github.com/bluenviron/mediamtx/issues/3582 "MediaMTX issue #3582 — relative versus absolute FFmpeg path"
[12]: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/where "Microsoft where command"
[13]: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/get-command?view=powershell-7.6 "Microsoft PowerShell Get-Command"
[14]: https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/paths/index.html "ffmpeg-sidecar paths module"
[15]: https://github.com/HandBrake/HandBrake/pull/6548 "HandBrake — harden DLL loading"
[16]: https://wiki.videolan.org/VLC_command-line_help/ "VideoLAN VLC command-line help"

## Atualização do ciclo v0.1.24 — 2026-08-20

A auditoria de follow-up confirmou que o risco não estava limitado ao `push_directory_candidates` do CWD. Mesmo após retirar a inserção explícita, `SearchPathW` com `lpPath = NULL` ainda podia retornar o diretório de trabalho conforme `SafeProcessSearchMode`, e `where.exe` também procura o diretório atual por padrão. Como o Rovex já enumera o PATH herdado e persistente, App Paths, diretório do executável, diretório adjacente explícito, raízes conhecidas e WinGet com limites, esses dois fallbacks não acrescentavam uma fonte necessária que justificasse a ambiguidade.

A implementação v0.1.24 removeu `windows_search_path` e `windows_where_candidates`. `backend_candidates` não adiciona mais o CWD implicitamente; `is_backend_file` continua exigindo caminho absoluto e arquivo regular após a resolução. Overrides absolutos e o diretório adjacente passado explicitamente pelo pipeline continuam permitidos. O gate `scripts/test_ffmpeg_discovery_contract.sh` impede a reintrodução de `current_directory`, `SearchPathW` ou `windows_where_candidates`, e um teste unitário confirma a ausência do CWD implícito quando ele não está declarado no PATH.

A decisão não transforma qualquer instalação em confiável: executáveis encontrados no PATH, Registro ou diretórios de usuário ainda exigem a política de confiança documentada e não são autenticados por assinatura ou hash. O próximo ciclo deve avaliar autenticação de origem, DLLs dependentes e contenção de processos descendentes.
