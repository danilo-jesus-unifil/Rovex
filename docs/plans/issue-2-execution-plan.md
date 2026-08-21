# Plano de execução do issue #2

**Issue:** [#2: uma prompt de ajuda](https://github.com/danilo-jesus-unifil/Rovex/issues/2)
**Estado:** aberto
**Método:** executar uma fase por vez, validar e registrar evidências antes de avançar.

## Regras de execução

O Rovex é um cliente desktop de filesystem para Windows 10/11. A ordem de prioridade é segurança, integridade dos dados, estabilidade, correção, acessibilidade, desempenho, UX e novas features. Implementações reais não devem ser substituídas por mocks, e uma feature só pode ser declarada pronta com testes e evidências.

A UI emite comandos e eventos; o core decide; adapters executam; workers processam; e resultados retornam como eventos tipados. Cada feature deve definir falhas, cancelamento, recuperação, testes, comportamento em Windows 10/11, custo de CPU e memória e possíveis efeitos sobre corrupção, concorrência, bloqueio da UI ou resultados obsoletos.

## Fases do issue

| Fase | Escopo | Critério de avanço |
|---|---|---|
| 0 | Ler o issue, preservar o contexto e decompor o trabalho | Plano registrado; nenhuma implementação prematura |
| 1: Foundation | Auditar arquitetura, filesystem, operações, UI, workers, testes e compatibilidade Windows | Auditoria de engenharia, roadmap P0-P3 e validações da base |
| 2: Core Explorer | Clipboard, drag-and-drop, Lixeira, propriedades, teclado, ocultos, ordenação e modos de visualização | Cada feature isolada, testada e documentada |
| 3: Search | Busca recursiva, filtros, engine e índice opcional | Busca cancelável, limitada e sem bloquear a UI |
| 4: Preview | Thumbnails, preview, metadados e hashing | Parsers seguros e limites de memória e tempo |
| 5: Advanced Tools | Duplicatas, armazenamento, compactados e operações em lote | Operações controladas, canceláveis e com integridade |
| 6: Windows Integration | Open With, Terminal, Shell integration, Lixeira e menu contextual | Adapters isolados e check Windows 10/11 |
| 7: Distribution | Installer, portable, assinatura, atualização e release | Artefatos verificáveis, checksums e documentação reproduzível |

## Trabalho concluído

A auditoria Foundation e o roadmap P0-P3 estão em [`../audits/ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md`](../audits/ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md). Os lotes abaixo foram implementados sobre os módulos existentes.

| Lote | Implementação e evidência |
|---|---|
| P0: manifesto | `assets/rovex.manifest`, integração no `build.rs` e validação no PE por `scripts/verify_windows_manifest.sh`. |
| P0: filesystem | Testes adversariais em `src/filesystem.rs` e `src/operations/tests.rs` para Unicode, espaços, pontuação, caminhos longos no host e nomes reservados condicionados ao Windows. |
| P1: teclado | Atalhos implementados e exercitados por `scripts/test_keyboard_shortcuts.sh`. |
| P1: ordenação | `src/desktop/state/sorting.rs` integrado ao filtro e ao loader; colunas de nome, tipo, tamanho, modificação, criação e acesso; `scripts/test_sorting.sh`. |
| P1: ocultos | `ListingOptions`, `symlink_metadata`, atributos Windows condicionados e `scripts/test_hidden_files.sh`; o toggle recarrega a pasta sem seguir links. |
| P1: Nova pasta | `create_directory`, confirmação, worker, refresh, Ctrl+Shift+N e `scripts/test_new_folder.sh`, com criação real de nome com espaço. |
| P1: clipboard | `copypasta` 0.10.2, payload tipado Copy/Cut, Ctrl+C/Ctrl+X/Ctrl+V e scheduler existente; `scripts/test_clipboard.sh` confirmou Cut/Paste no filesystem. |
| P1: Propriedades | Diálogo somente leitura baseado em `LoadedRow`, uma seleção, sem seguir links/reparse e com Flickable; `scripts/test_properties.sh` confirmou rolagem e preservação do arquivo. |
| P1: drag-and-drop | `DropArea`/`DataTransfer` para payloads internos e `WinitWindowAccessor` para `HoveredFile`/`DroppedFile`; o drop usa `PathBuf` e despacha `Copy` pelo scheduler. O arraste efetivo do Explorer continua pendente de execução nativa. |
| Search | Filtro local separado do botão Buscar; traversal recursivo em worker, batches, geração anti-stale, cancelamento e refresh/navegação que cancelam a busca. `scripts/test_recursive_search.sh` confirmou níveis profundos. Indexação nativa, árvores grandes reais e execução nativa Windows continuam pendentes. |
| Preview | `image` 0.25.10 em worker; decode por conteúdo para BMP/GIF/JPEG/PNG/WebP, rejeição de symlink/reparse e não regulares, limites de 128 MiB de entrada, 8192x8192, 64 MiB de decode e thumbnail de 256 px. Cache LRU: 128 entradas e 32 MiB. `scripts/test_preview.sh` confirmou PNG válido, JPG inválido e fechamento do painel. |
| Text Preview | UTF-8, UTF-8 com BOM e UTF-16LE/BE com BOM; leitura máxima de 64 KiB, remoção de BOM, rejeição de NUL/controles/binário e indicação de truncamento. PDF, vídeo, áudio, Office, handlers COM/Explorer e codecs externos não são ativados no processo principal. |
| Settings | `src/settings/`, arquivo v1 em `%LOCALAPPDATA%\\Rovex` no Windows, path hexadecimal nativo, parser limitado, escrita temporária sincronizada e `MoveFileExW` com replace/write-through. `scripts/test_settings.sh` confirmou persistência na segunda inicialização. |
| Recycle Bin | Adapter Windows com `SHFileOperationW`, UTF-16, `FOF_ALLOWUNDO`, `FOF_NORECURSION`, confirmação única e erro estruturado; a evolução para `IFileOperation`/COM veio no lote seguinte. O Linux mantém a exclusão permanente de desenvolvimento. |
| Distribution | `scripts/package_windows_portable.sh` e `scripts/verify_windows_portable.sh`; v0.1.11 publicou ZIP e SHA-256, e v0.1.12 consolidou manifesto e documentação de assinatura ausente. |
| Process Hardening | Cleanup centralizado com `kill`/`wait`/join, `Command::arg`, stdin fechado e testes fake de timeout/cancelamento. O smoke JXL em diretórios separados voltou a passar após correção de coordenadas e da opção incompatível no ffprobe. |
| v0.1.13 | Runner `windows-latest` validou testes, Clippy, release e `scripts/verify_windows_native.ps1`. `validate_destination` passou a rejeitar nomes reservados; o round-trip de settings usa caminho absoluto; e o replace Windows foi serializado para evitar `PermissionDenied`. Release portable publicada. |
| v0.1.14 | Adapter Shell evoluído para `IFileOperation`/COM com `SHCreateItemFromParsingName`, `IShellItem`, `FOFX_RECYCLEONDELETE` e `FOFX_EARLYFAILURE`; fallback para `SHFileOperationW` somente antes de uma mutação parcial. |
| v0.1.15 | `Abrir Terminal aqui`: `wt.exe`, PowerShell e `cmd.exe` em cascata, com `--startingDirectory`, argumentos separados e `current_dir`; não usa `cmd /c` nem `powershell -Command`. O worker é nomeado, o botão fica desabilitado no Linux e os testes cobrem seleção de diretório. |
| v0.1.16 | `Abrir com...` via `SHOpenWithDialog`/`OPENASINFO` no Windows; uma seleção de arquivo regular, buffer UTF-16, `pcszClass = NULL`, `OAIF_EXEC`, worker COM STA e sem `runas`, `rundll32`, associação automática ou shell concatenado. |

## Atualizações de auditoria

### v0.1.18: ativação explícita de arquivos

`activate(int)` passou a abrir arquivos regulares por `ShellExecuteExW` com verbo padrão em worker COM STA. `Abrir com...` permanece separado. Caminhos relativos, ausentes, diretórios, `..`, symlinks e reparse points no alvo ou nos pais são recusados. Foram adicionados cinco testes de ativação, fixtures de pai symlinkado e caminho ambíguo, `scripts/test_activation_contract.sh` e smoke visual.

### v0.1.19: erros de ativação

O adapter passou a usar `SEE_MASK_NOASYNC` e `COINIT_DISABLE_OLE1DDE` no worker sem message loop. `hInstApp`, códigos `SE_ERR_*` e `GetLastError` são preservados em mensagens controladas. O gate verifica flags, COM, ponteiros nulos e ausência de `Command::new`. Associação inexistente, arquivo bloqueado, junction, UNC e path longo ainda exigem execução nativa.

### v0.1.20: caminhos longos

O manifesto já declarava `longPathAware`, mas o smoke não atravessava uma árvore acima de 260 caracteres. `scripts/verify_windows_native.ps1` passou a criar quatro níveis reais, medir o caminho e executar `cargo run --quiet -- --cli` nessa pasta. `scripts/test_windows_native_contract.sh` exige o manifesto, o fixture e a chamada no CI. UNC, `\\?\\`, junctions, mounted folders, ACLs e associação inexistente continuam pendentes.

### v0.1.21: reparse points

A listagem passou a verificar `FILE_ATTRIBUTE_REPARSE_POINT` na raiz e nas entradas antes de `is_dir()`/`read_dir`; entradas reparse são classificadas como `EntryKind::Symlink`. O smoke Windows cria uma junction com `mklink /J` e exige erro controlado. Também foi corrigida a corrida do teste de cancelamento FFmpeg com handshake de readiness. O smoke passou a capturar o retorno não-zero esperado em `junctionExitCode` e zerar `LASTEXITCODE` antes de concluir.

### v0.1.22: nomes reservados

`is_reserved_windows_name` deixou de depender apenas de `is_ascii_digit()`. A lista Windows inclui `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `COM1`-`COM9`, `COM¹`-`COM³`, `LPT1`-`LPT9` e `LPT¹`-`LPT³`, inclusive com extensões. `scripts/test_reserved_windows_names_contract.sh` exige esses nomes no predicado, fixture e workflow.

### v0.1.23: auditoria do núcleo

Foram corrigidos quatro casos: raiz reparse na busca, colisão de extensão por caixa no Windows, pais reparse em destinos e junction final entrando no fluxo de diretório da exclusão. A busca recebeu `SearchError::RootRedirected`; `same_existing_path` usa canonicalização Windows; destinos usam helper único de reparse; a exclusão trata junction final como link. O risco de CWD na descoberta de FFmpeg/ffprobe foi documentado, mas não alterado sem fixture adversarial nativa.

### v0.1.24: descoberta de FFmpeg/ffprobe

A descoberta deixou de inserir o CWD implicitamente e o adapter Windows deixou de usar `SearchPathW(lpPath = NULL)` e `where.exe`. A decisão considera os contratos de `SearchPathW` e `CreateProcess`. Permanecem overrides absolutos, PATH herdado e persistente, App Paths, diretório do executável, diretório adjacente explícito, raízes conhecidas e pacotes WinGet com limites. `is_backend_file` exige caminho absoluto e arquivo regular. O teste `descoberta_nao_adiciona_cwd_implicitamente` e o gate `scripts/test_ffmpeg_discovery_contract.sh` cobrem a regra.

### v0.1.25: contenção de processos

A reprodução Unix confirmou que matar somente o processo direto não encerra descendentes que mantêm stdout/stderr abertos. Unix usa grupos e `killpg`; Windows usa Job Objects com `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, `AssignProcessToJobObject` e `TerminateJobObject`. `windows-sys` recebeu apenas as features nativas necessárias e `libc` foi fixado em `0.2.189` para Unix. A associação falha fechada se não puder ser estabelecida. O teste é `cancelamento_encerra_descendente_que_mantem_pipe_aberto` e o contrato é `scripts/test_process_containment_contract.sh`.

### v0.1.26: fallback de terminação

`ProcessTree::terminate` passou a retornar `io::Result<()>`. Quando `killpg` ou `TerminateJobObject` falha, o código tenta `Child::kill` antes de aguardar o processo direto. O fallback não é apresentado como contenção completa.

### v0.1.27: reserva atômica

`temporary_path` passou de `exists()` para `OpenOptions::create_new(true)`, criando e fechando um placeholder antes do spawn. Colisões avançam para a próxima tentativa; outros erros de I/O são preservados. O teste `reserva_de_temporario_e_atomica_e_cria_placeholder` confirma caminhos distintos e placeholders existentes.

### v0.1.28: placeholder até o spawn

O pipeline deixou de remover o placeholder antes do spawn. A reserva permanece durante spawn, validação e retries. Como `-n` recusava uma saída já existente, FFmpeg usa `-y` somente sobre o temporário privado, enquanto a publicação final continua sem sobrescrita. Foram adicionados `ffmpeg_pode_sobrescrever_placeholder_temporario_reservado` e `scripts/test_converter_temporary_contract.sh`. A suíte passou com 109 testes.

## Formato exigido pelo issue

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

## Workflow e critérios de pronto

Cada lote deve seguir: inspecionar o módulo; definir risco e aceite; implementar mudança pequena; executar `cargo fmt`, `cargo check`, testes focados e validações de plataforma; revisar segurança, concorrência, desempenho, UX e acessibilidade; atualizar a documentação; criar commit descritivo; e só então avançar.

Uma feature só está pronta quando tem implementação real, erro estruturado, cancelamento quando aplicável, comportamento seguro de recuperação, testes relevantes, documentação atualizada, validação Windows 10/11 quando aplicável e evidência de que resultados obsoletos não substituem o estado atual.
