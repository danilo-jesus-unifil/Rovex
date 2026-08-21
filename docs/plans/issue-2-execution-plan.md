# Plano de execução do issue #2

**Issue:** [#2 — uma prompt de ajuda](https://github.com/danilo-jesus-unifil/Rovex/issues/2)
**Estado:** aberto
**Método:** executar uma fase por vez, validar e registrar evidências antes de avançar.

## Regras que governam toda a execução

O issue define o Rovex como um filesystem client desktop para Windows 10/11, não apenas como uma interface. A prioridade obrigatória é segurança, integridade dos dados, estabilidade, correção, acessibilidade, desempenho, UX e somente depois novas features. Nenhuma implementação deve substituir funcionalidades reais por mocks, duplicar subsistemas existentes sem necessidade ou declarar uma feature pronta sem testes e evidências.

A UI deve emitir comandos/eventos; o core deve decidir; adapters devem executar; workers devem processar; e os resultados devem voltar como eventos tipados. Cada nova feature deverá responder como funciona, como falha, como cancela, como recupera, como é testada, como se comporta no Windows 10/11, qual custo de CPU/memória e se pode gerar corrupção, race condition, bloqueio da UI ou resultado obsoleto.

## Fases do master prompt

| Fase | Escopo do issue | Critério para avançar |
|---|---|---|
| 0 | Ler o issue, preservar o contexto e decompor o trabalho | Plano registrado e nenhuma implementação prematura |
| 1 — Foundation | Auditar arquitetura, filesystem, operações, UI, workers, testes e compatibilidade Windows; corrigir problemas encontrados | Auditoria `ROVEX ENGINEERING AUDIT` completa, roadmap P0–P3 e validações da base |
| 2 — Core Explorer | Clipboard, drag/drop, recycle bin, propriedades, navegação por teclado, ocultos, ordenação e modos de visualização | Cada feature isolada, implementada sobre módulos existentes, testada e documentada |
| 3 — Search | Busca recursiva, filtros, engine de busca e índice opcional | Busca cancelável, sem travar UI, com limites e testes de caminhos/erros |
| 4 — Preview | Thumbnails, painel de preview, metadados e hashing | Parsers seguros, limites de memória/tempo, sem execução de conteúdo não confiável |
| 5 — Advanced Tools | Duplicatas, análise de armazenamento, arquivos compactados e operações em lote | Operações controladas, canceláveis e com tratamento de integridade |
| 6 — Windows Integration | Open With, Terminal, Shell integration, Recycle Bin e integração contextual | Check Windows 10/11 e adapters isolados de plataforma |
| 7 — Distribution | Installer, portable, assinatura, atualização e pipeline de release | Artefatos verificáveis, checksums, documentação e release reproduzível |

## Próxima etapa isolada

A auditoria Foundation e o roadmap P0–P3 já foram produzidos em `../audits/ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md`. O primeiro lote P0, manifesto Windows, foi implementado em `assets/rovex.manifest`, integrado pelo `build.rs` e validado no PE release com `scripts/verify_windows_manifest.sh`. O segundo lote P0, testes adversariais de filesystem, foi implementado em `src/filesystem.rs` e `src/operations/tests.rs`; a suíte cobre Unicode, espaços, pontuação, caminhos longos no host e nomes reservados condicionados ao Windows. O primeiro lote P1, atalhos de teclado, foi implementado na UI e exercitado sob Xvfb com `scripts/test_keyboard_shortcuts.sh`. O segundo lote P1, ordenação, foi implementado em `src/desktop/state/sorting.rs`, integrado ao filtro e ao loader, com colunas de nome, tipo, tamanho, modificação, criação e acesso; `scripts/test_sorting.sh` exercita os cabeçalhos. O terceiro lote P1, arquivos ocultos e de sistema, foi implementado com `ListingOptions`, `symlink_metadata`, atributos Windows condicionados e `scripts/test_hidden_files.sh`; o toggle recarrega a pasta sem seguir links nem disparar operações. A suíte passou no host, os smoke tests gráficos passaram e o código compila para Windows GNU. A execução nativa Windows 10/11, incluindo atributos NTFS reais, acessibilidade, DPI e reparse points, continua pendente; nenhuma compatibilidade nativa é declarada sem essa evidência. O quarto lote P1, Nova pasta, foi implementado sobre `create_directory`, com validação de nome, confirmação, worker existente, refresh e atalho Ctrl+Shift+N; `scripts/test_new_folder.sh` verifica a criação real de um nome com espaço no filesystem sob Xvfb. O quinto lote P1, clipboard, usa `copypasta` 0.10.2 como adapter persistente, payload tipado Copy/Cut, Ctrl+C/Ctrl+X/Ctrl+V e o scheduler de operações para Paste; `scripts/test_clipboard.sh` confirmou Cut/Paste real no filesystem sob Xvfb. A suíte, Clippy, auditoria, deny, cross-build Windows e smoke tests passaram nos lotes. A execução nativa Windows 10/11, acessibilidade e interoperabilidade com Explorer/CF_HDROP continuam pendentes; nenhuma dessas compatibilidades é declarada sem evidência. O sexto lote P1, Propriedades, foi implementado como diálogo somente leitura acionado pelo menu contextual. O handler reutiliza o snapshot `LoadedRow` já publicado, exige exatamente uma seleção, não segue links/reparse points e não relê ou altera o filesystem; o corpo usa Flickable para conter General, Security e Details. `scripts/test_properties.sh` abriu o diálogo para nome Unicode, exercitou a rolagem, confirmou que o arquivo permaneceu intacto e que o processo continuou ativo. A execução nativa Windows 10/11, ACLs reais, atributos completos, reparse points e escala de acessibilidade continuam pendentes. O sétimo lote P1, drag-and-drop, foi implementado com `DropArea`/`DataTransfer` para payloads internos e filtro `WinitWindowAccessor` para `HoveredFile`/`DroppedFile`. O hover valida arquivo absoluto regular, o drop usa `PathBuf` diretamente e despacha `Copy` pelo scheduler existente, com progresso, cancelamento e refresh; `scripts/test_*`, cross-build e manifesto passaram, enquanto o arraste efetivo do Explorer segue pendente de execução nativa. O próximo item isolado será busca recursiva, com engine cancelável e limites de filesystem antes de publicar resultados. O oitavo lote Search foi implementado: o filtro local permaneceu separado, o botão Buscar iniciou traversal recursivo em worker dedicado, resultados chegaram em batches com geração anti-stale, Cancelar interrompeu cooperativamente e refresh/navegação também cancelaram a busca. Testes unitários cobrem ordem determinística, case-insensitive, ocultos, limites, raiz relativa, symlink e coalescência; `scripts/test_recursive_search.sh` confirmou resultados em níveis profundos sob Xvfb. Indexação nativa do Windows, grandes árvores reais e execução nativa Windows 10/11 continuam pendentes. O próximo item isolado será thumbnails/preview, com parsers seguros, limites de memória/tempo e fallback para ícone genérico. O nono lote Preview foi implementado com `image` 0.25.10 em worker dedicado: decode por conteúdo para BMP/GIF/JPEG/PNG/WebP, rejeição de symlink/reparse point e arquivos não regulares, limites de 128 MiB de entrada, 8192×8192 de dimensão, 64 MiB de decode e thumbnail de 256px. O cache LRU é limitado a 128 entradas/32 MiB; gerações cancelam requests antigos, seleção múltipla/pastas/navegação escondem o painel e falhas usam fallback sem crash. `scripts/test_preview.sh` confirmou PNG válido, `.jpg` inválido e fechamento do painel; a matriz completa, Clippy, audit, deny, cross-check/build Windows GNU, manifesto e smoke tests passaram. O décimo lote Text Preview reutiliza o mesmo worker/cache e tenta apenas conteúdo literal: UTF-8, UTF-8 com BOM e UTF-16LE/BE com BOM, lendo no máximo 64 KiB, removendo BOM, rejeitando NUL/controles/binário e marcando truncamento. O Flickable escuro mostra Unicode sem links nem execução; testes unitários cobrem as duas endiannesses, BOM, binários, UTF-8 inválido, truncamento, arquivo vazio, erro de I/O e symlink; o smoke confirmou `nota.txt` no painel. PDF, vídeo, áudio, Office, handlers COM/Explorer e codecs externos continuam pendentes e não são ativados no processo principal. O décimo primeiro lote Settings foi implementado em `src/settings/`: o arquivo v1 fica em `%LOCALAPPDATA%\\Rovex` no Windows, sem privilégios, usa path hexadecimal nativo, parser limitado com fallback seguro e escrita temporária sincronizada; `MoveFileExW` aplica replace/write-through no alvo Windows. Startup restaura último diretório válido, Ocultos e coluna/direção de ordenação; os handlers salvam alterações imediatamente e o encerramento tenta salvar novamente. Testes unitários cobrem Unicode, replace sem temporário órfão, arquivo ausente/corrompido, limite de 16 KiB e chaves futuras; `scripts/test_settings.sh` alternou preferências e confirmou a segunda inicialização ativa. A execução nativa Windows 10/11, ACLs e redirecionamento de perfil permanecem pendentes. Preview de PDF, vídeo, áudio e Office continua fora do processo principal e requer workers isolados. O décimo segundo lote Recycle Bin foi implementado no alvo Windows com adapter `SHFileOperationW` UTF-16 e `FOF_ALLOWUNDO`, `FOF_NORECURSION`, confirmação única na UI, erro estruturado e nenhum fallback silencioso para exclusão permanente. A validação existente de raiz, symlink e diretórios não vazios foi preservada; o Linux mantém a exclusão permanente de desenvolvimento. O cross-check Windows GNU, testes host, Clippy e formatação passaram, mas Lixeira real, restauração pelo Explorer, ACLs, UNC/SMB, volumes removíveis e paths longos continuam pendentes de execução nativa. A API moderna `IFileOperation`/COM permanece evolução posterior. O décimo terceiro lote Distribution foi implementado em `scripts/package_windows_portable.sh` e `scripts/verify_windows_portable.sh`: a v0.1.11 publicou ZIP portable, manifesto e SHA-256, e a v0.1.12 consolidou o pipeline com documentação de assinatura ausente e checksum verificável. O lote seguinte Process Hardening centralizou `kill`/`wait`/join, manteve `Command::arg` sem shell, fechou stdin e criou testes fake de timeout/cancelamento; o smoke de JPEG XL em diretórios separados voltou a passar após corrigir coordenadas obsoletas e o uso incompatível de `-nostdin` no ffprobe. O décimo quarto lote, v0.1.13, fechou o primeiro ciclo de validação nativa não interativa: o runner `windows-latest` executou testes, Clippy, build release e `scripts/verify_windows_native.ps1` com Unicode e espaços após corrigir três falhas reais. `validate_destination` agora rejeita nomes reservados Windows (`CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `COM1`–`COM9` e `LPT1`–`LPT9`, inclusive variantes com extensão/pontuação); o teste de round-trip de settings usa caminho absoluto nativo; e o replace atômico é serializado por mutex de processo para evitar `PermissionDenied` em `MoveFileExW` concorrente. A matriz local completa, incluindo host/cross Clippy, audit/deny, builds, manifesto e todos os smoke tests, passou; a release portable v0.1.13 foi publicada com ZIP e SHA-256. Essa evidência não substitui Windows 10/11 interativo: drag-and-drop do Explorer, Lixeira real, ACLs, DPI, acessibilidade, reparse points, UNC/SMB e volumes removíveis continuam pendentes. O décimo quinto lote, preparado para a v0.1.14, evoluiu o adapter Shell para `IFileOperation`/COM. O Windows usa `SHCreateItemFromParsingName`, `IShellItem`, `FOFX_RECYCLEONDELETE` e `FOFX_EARLYFAILURE` em um worker, com bindings ABI mínimos isolados; falhas de COM, parsing ou enfileiramento antes de `PerformOperations` usam `SHFileOperationW` com `FOF_ALLOWUNDO`, enquanto falhas após o início não são repetidas. Testes de UTF-16, GUIDs oficiais, flags, HRESULT estruturado e regra de fallback foram adicionados. Host tests, Clippy host/cross, cross-check, link de testes Windows, audit/deny, builds release, manifesto e todos os smoke tests passaram; o CI `windows-latest` também passou a suíte nativa. A validação interativa de restauração da Lixeira, Windows 10 22H2, UNC/SMB, ACLs, volumes removíveis e paths extremos continua pendente. O décimo sexto lote implementou `Abrir Terminal aqui` como ação contextual explícita. A seleção de pasta abre nela; a seleção de arquivo usa o diretório pai; caminhos relativos, ausentes e alvos inválidos são rejeitados. No Windows, a cascata usa `wt.exe` com `--startingDirectory`, depois PowerShell e por fim `cmd.exe`, sempre com argumentos separados e `current_dir`, sem `cmd /c`, `powershell -Command` ou shell concatenado. O worker nomeado devolve o resultado ao event loop Slint; no Linux o botão fica desabilitado. Testes unitários cobrem Unicode, espaços, diretórios, arquivo/pai, alvos inválidos e separação do caminho como argumento; captura Xvfb confirmou o botão no tema escuro e o menu rolável. O CI Windows ainda precisa validar visualmente cada candidato em sessão interativa. O décimo sétimo lote implementou `Abrir com...` via `SHOpenWithDialog`/`OPENASINFO` no Windows. O callback aceita somente uma seleção de arquivo regular; pastas, links, reparse points, caminhos relativos e itens ausentes são recusados. A chamada usa buffer UTF-16, `pcszClass = NULL` e somente `OAIF_EXEC`, sem flags de associação padrão, sem `runas`, sem `rundll32`, sem shell concatenado e sem execução ao listar a pasta. COM STA e o diálogo executam em worker nomeado, e o resultado retorna ao event loop Slint. Testes host cobrem Unicode, espaços, arquivo/pasta, inexistente e symlink; cross-check e link Windows validam Shell32/ABI; smoke Xvfb confirma a posição e o estado disabled no Linux. O CI Windows ainda não abre uma sessão visual para validar a seleção de aplicativo em Windows 10/11. O próximo item isolado será consolidar ativação explícita de arquivos e verbs Shell somente quando houver contrato claro para não confundir Open With, associação padrão e execução automática.

O formato exigido pelo issue para a auditoria foi registrado como:

```text
ROVEX ENGINEERING AUDIT
1. Current architecture
2. Existing modules
3. Existing features
4. Missing features
5. Technical debt
6. Security risks
7. Performance risks
8. Windows 10 risks
9. Windows 11 risks
10. UI/UX gaps
11. Accessibility gaps
12. Testing gaps
13. Documentation gaps
14. Dependency risks
15. Recommended implementation order

ROADMAP
P0 — Critical
P1 — High
P2 — Medium
P3 — Experimental
```

Nenhuma feature da Fase 2 ou posterior será marcada como iniciada antes de essa auditoria ser concluída e validada. Problemas P0 descobertos na auditoria são tratados em mudanças pequenas, com branch/commit próprio e validação incremental; o manifesto Windows é o primeiro lote concluído sob essa regra.

## Atualização do ciclo v0.1.18 — 2026-08-20

O lote de **ativação explícita de arquivos** foi concluído após confirmação de uma lacuna real: `activate(int)` navegava apenas para diretórios e não havia ação contextual `Abrir`. A implementação usa `ShellExecuteExW` com verbo padrão em worker COM STA, mantém `Abrir com...` separado e recusa caminhos relativos, ausentes, diretórios, `..`, symlinks e reparse points no alvo ou em componentes pais.

As validações foram incrementadas com cinco testes de ativação no host, fixture de arquivo dentro de pai symlinkado, teste de caminho ambíguo, gate `scripts/test_activation_contract.sh` no CI Ubuntu/Windows, check/Clippy/build release Windows GNU e smoke visual do menu contextual/Open With. O primeiro Clippy cross revelou e permitiu corrigir `field_reassign_with_default`; a execução seguinte passou.

Próximos riscos a investigar, sem declarar falha antes de reproduzir: associação padrão inexistente (`SE_ERR_NOASSOC`), arquivo bloqueado/sem permissão no Windows, junctions e outros reparse tags não-symlink, caminhos UNC/longos e comportamento efetivo do verbo padrão em Windows 10 e 11. Cada caso deve gerar fixture ou teste de regressão antes de qualquer correção.

## Atualização do ciclo v0.1.19 — 2026-08-20

A investigação confirmou uma lacuna independente da associação do usuário: o adapter chamava `ShellExecuteExW` em worker sem message loop com `fMask = 0`, embora a documentação exija `SEE_MASK_NOASYNC` nesse cenário. Também confirmou que o contrato COM recomendado inclui `COINIT_DISABLE_OLE1DDE`. O lote foi corrigido sem adicionar fallback de shell.

O erro agora preserva `hInstApp`/`SE_ERR_*` e `GetLastError`, com mensagens controladas para associação ausente, acesso negado, compartilhamento e cancelamento. A suíte subiu para 103 testes e o gate estrutural verifica flags, COM, ponteiros nulos e ausência de `Command::new`.

O próximo ciclo deve reproduzir em Windows nativo associação inexistente, arquivo bloqueado, junction/mounted folder, caminho UNC e path longo antes de qualquer nova mudança. Nenhum desses casos é declarado resolvido apenas por cross-build.

## Atualização do ciclo v0.1.20 — 2026-08-20

A auditoria confirmou uma lacuna de validação, não uma falha funcional presumida: `assets/rovex.manifest` já declara `longPathAware`, porém `verify_windows_native.ps1` não criava nem listava uma árvore acima de 260 caracteres. O smoke foi ampliado para criar quatro níveis reais, medir o caminho acima de MAX_PATH e executar o CLI nessa pasta.

O gate `scripts/test_windows_native_contract.sh` foi adicionado ao CI para impedir que o fixture, a declaração `longPathAware` ou a chamada Windows sejam removidos sem detecção. A pesquisa oficial registrou as diferenças entre DOS, UNC e `\\?\\` extended-length. UNC, junctions, mounted folders, ACLs e associação inexistente continuam sem declaração de suporte até haver fixtures nativas controladas.

## Atualização do ciclo v0.1.21 — 2026-08-20

A investigação confirmou uma falha de classificação: a listagem usava `FileType::is_symlink()` e `is_dir()` sem verificar explicitamente `FILE_ATTRIBUTE_REPARSE_POINT` na raiz. Como junctions são reparse points, a navegação podia depender de como a biblioteca classificava o tipo e chegar a `read_dir` sobre o destino. O código agora trata qualquer reparse point como não navegável antes de listar e classifica entradas reparse como `EntryKind::Symlink`.

O smoke Windows cria uma junction real para um diretório com marcador externo e exige erro controlado do CLI; o gate estrutural impede a remoção desse cenário. Também foi corrigida uma corrida real no teste de cancelamento FFmpeg: o cancelamento só ocorre após handshake de readiness, e não após um atraso fixo de 100 ms.

O primeiro CI confirmou ainda uma falha operacional no próprio smoke: o retorno não-zero esperado da junction ficou em `LASTEXITCODE` e fez o PowerShell terminar com código 1 apesar de a asserção de segurança passar. O script agora captura `junctionExitCode` e zera o estado antes de concluir; o gate estrutural exige essa limpeza.

## Atualização do ciclo v0.1.22 — 2026-08-20

A investigação confirmou uma lacuna no predicado de nomes reservados: `is_ascii_digit()` cobria `COM1`–`COM9` e `LPT1`–`LPT9`, mas não os dígitos sobrescritos ¹, ² e ³. A documentação oficial lista `COM¹`/`COM²`/`COM³` e `LPT¹`/`LPT²`/`LPT³` como dispositivos reservados, inclusive com extensões. O código agora mantém uma lista explícita dos nomes ASCII e sobrescritos, e a fixture Windows de operações cobre todos os casos.

O gate `scripts/test_reserved_windows_names_contract.sh` exige a presença dos seis nomes no predicado, no teste e no workflow. O próximo ciclo deve cobrir arquivos maiores que 4 GB quando o runner permitir, outras tags de reparse, mounted folders, arquivos bloqueados, ACLs, UNC e associação inexistente somente com fixtures nativas específicas.

## Atualização do ciclo v0.1.23 — 2026-08-20

A auditoria exploratória do estado v0.1.22 confirmou quatro inconsistências reais. A busca recursiva não rejeitava uma raiz reparse antes de `read_dir`; a saída de conversão comparava caminhos apenas lexicalmente e podia perder colisões de caixa no Windows; destinos podiam atravessar pais junction/reparse porque a política reconhecia symlink, mas não o atributo `FILE_ATTRIBUTE_REPARSE_POINT`; e a exclusão podia tratar um junction final como diretório e inspecionar seu conteúdo.

O lote corrigiu os quatro casos em módulos pequenos: `SearchError::RootRedirected` para a raiz da busca, canonicalização Windows em `same_existing_path`, helper único de reparse points para pais de destino e classificação de reparse final como link no fluxo de exclusão. Foram adicionados testes host e Windows condicionados, incluindo junction criada por `mklink /J`; a suíte ficou em 105 testes host aprovados e 2 ignorados explicitamente. Check, Clippy host/cross, build release Windows GNU, audit/deny, contratos nativos e layout documental passaram.

A mesma auditoria confirmou, por leitura do contrato de `SearchPathW`/`CreateProcess`, que o diretório atual aparece entre candidatos de FFmpeg/ffprobe. Como overrides absolutos e instalações de gerenciadores são suportados, a mudança foi registrada como decisão de confiança e não aplicada sem fixture adversarial nativa. O próximo lote deve testar um backend falso no CWD, decidir a ordem/política e preservar a regra de não baixar executáveis. TOCTOU baseado em caminho, Job Objects, UNC/SMB, ACLs, arquivos bloqueados e Windows interativo continuam pendentes e não são declarados resolvidos.

## Atualização do ciclo v0.1.24 — 2026-08-20

A auditoria de follow-up confirmou que remover somente a inserção explícita do CWD não era suficiente: `SearchPathW` com `lpPath = NULL` e `where.exe` também podiam consultar o diretório de trabalho. A correção removeu esses dois fallbacks do adapter Windows. A descoberta continua determinística por override absoluto, PATH herdado, PATH persistente, App Paths, diretório do executável, diretório adjacente explicitamente fornecido, raízes conhecidas e pacotes WinGet com limites; `is_backend_file` exige caminho absoluto e arquivo regular.

A validação foi incrementada com o teste `descoberta_nao_adiciona_cwd_implicitamente`, o contrato `scripts/test_ffmpeg_discovery_contract.sh` no job de qualidade e a refatoração dos testes de segurança para `src/security/tests.rs`, mantendo todos os arquivos Rust abaixo de 400 linhas. O ciclo precisa confirmar que a remoção não quebra instalação por PATH, override ou diretório adjacente e que os jobs Windows nativos continuam verdes.

O risco restante é de confiança dos próprios candidatos autorizados: PATH, Registro e diretórios de usuário não têm autenticação por hash/assinatura. Também permanecem TOCTOU baseado em caminho, DLLs carregadas pelo backend, descendentes sem Job Object, UNC/SMB, ACLs e Windows interativo. Nenhum desses pontos deve ser marcado como resolvido sem reprodução específica.

## Atualização do ciclo v0.1.25 — 2026-08-20

A auditoria confirmou por reprodução Unix que matar somente o processo direto não encerra necessariamente descendentes que herdam stderr/stdout. Como o worker faz `join` dos leitores dos pipes, um descendente pode manter o pipe aberto e atrasar cancelamento ou timeout até sair naturalmente. A correção criou `src/converters/process_tree.rs`: grupos de processos e `killpg` em Unix; Job Objects com `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, `AssignProcessToJobObject` e `TerminateJobObject` no Windows.

O teste `cancelamento_encerra_descendente_que_mantem_pipe_aberto` reproduz a condição com um backend fake que cria `sleep` em background; o contrato `scripts/test_process_containment_contract.sh` verifica que a política e o teste permanecem conectados ao CI. O módulo `windows-sys` recebeu apenas as features nativas necessárias e `libc` foi fixado em `0.2.189` para Unix.

A correção é contenção de árvore, não sandbox: a associação ocorre logo após o spawn e falha fechada se Job Object não puder ser criado/configurado/associado. DLLs dependentes, autenticação de executável, corrida entre spawn e associação, processos que tentam breakaway, TOCTOU de filesystem, ACLs, UNC/SMB e Windows interativo continuam exigindo validação nativa específica.

## Atualização do ciclo v0.1.26 — 2026-08-20

A revisão pós-v0.1.25 identificou que `ProcessTree::terminate` ignorava falhas de `killpg`/`TerminateJobObject` antes de `wait`. Isso não foi declarado como uma nova reprodução Windows; foi validado como uma lacuna de tratamento de erro no caminho recém-adicionado. O helper agora examina o resultado, tenta `Child::kill` como fallback e mantém o comportamento de falha fechada quando o Job Object não pôde ser estabelecido.

A suíte de processos e o contrato de contenção foram executados novamente. O próximo ciclo deve testar nativamente a associação dentro de outro Job Object, falhas induzidas de terminação, breakaway e a janela entre spawn e associação; não assumir que o fallback direto contém descendentes.

## Atualização do ciclo v0.1.27 — 2026-08-20

A auditoria de conversão confirmou que `temporary_path` apenas consultava `exists()` antes de devolver o nome. Essa não era uma reserva; duas conversões concorrentes poderiam iniciar FFmpeg no mesmo temporário. A implementação passou a usar `OpenOptions::create_new(true)`, cria e fecha um placeholder antes do spawn, avança somente em `AlreadyExists` e preserva erros reais de I/O. O teste `reserva_de_temporario_e_atomica_e_cria_placeholder` comprova a exclusividade de duas reservas consecutivas.

A validação agora inclui o placeholder atômico, a contagem de 108 testes, dez repetições paralelas da suíte e o gate de contenção mantido. A próxima investigação deve separar reserva de caminho de proteção por handle, sem declarar TOCTOU residual como resolvido sem fixture específica.

## Workflow de cada lote

Cada lote seguirá o ciclo: inspecionar o módulo existente; definir risco e critério de aceite; implementar uma mudança pequena; executar `cargo fmt`, `cargo check`, testes focados e validações de plataforma pertinentes; revisar segurança, concorrência, desempenho, UX e acessibilidade; atualizar documentação; criar commit descritivo; e somente então avançar.

## Critérios de pronto

Uma feature só será considerada pronta quando houver implementação real, fluxo de erro estruturado, cancelamento quando aplicável, recuperação ou comportamento seguro, testes relevantes, documentação atualizada, validação Windows 10/11 quando aplicável e evidência de que resultados obsoletos não substituem o estado atual.
