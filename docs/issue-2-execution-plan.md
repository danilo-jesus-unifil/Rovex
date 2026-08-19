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

A auditoria Foundation e o roadmap P0–P3 já foram produzidos em `ROVEX-ENGINEERING-AUDIT-ISSUE-2-2026-08-18.md`. O primeiro lote P0, manifesto Windows, foi implementado em `assets/rovex.manifest`, integrado pelo `build.rs` e validado no PE release com `scripts/verify_windows_manifest.sh`. O segundo lote P0, testes adversariais de filesystem, foi implementado em `src/filesystem.rs` e `src/operations/tests.rs`; a suíte cobre Unicode, espaços, pontuação, caminhos longos no host e nomes reservados condicionados ao Windows. O primeiro lote P1, atalhos de teclado, foi implementado na UI e exercitado sob Xvfb com `scripts/test_keyboard_shortcuts.sh`. O segundo lote P1, ordenação, foi implementado em `src/desktop/state/sorting.rs`, integrado ao filtro e ao loader, com colunas de nome, tipo, tamanho, modificação, criação e acesso; `scripts/test_sorting.sh` exercita os cabeçalhos. O terceiro lote P1, arquivos ocultos e de sistema, foi implementado com `ListingOptions`, `symlink_metadata`, atributos Windows condicionados e `scripts/test_hidden_files.sh`; o toggle recarrega a pasta sem seguir links nem disparar operações. A suíte passou no host, os smoke tests gráficos passaram e o código compila para Windows GNU. A execução nativa Windows 10/11, incluindo atributos NTFS reais, acessibilidade, DPI e reparse points, continua pendente; nenhuma compatibilidade nativa é declarada sem essa evidência. O quarto lote P1, Nova pasta, foi implementado sobre `create_directory`, com validação de nome, confirmação, worker existente, refresh e atalho Ctrl+Shift+N; `scripts/test_new_folder.sh` verifica a criação real de um nome com espaço no filesystem sob Xvfb. O quinto lote P1, clipboard, usa `copypasta` 0.10.2 como adapter persistente, payload tipado Copy/Cut, Ctrl+C/Ctrl+X/Ctrl+V e o scheduler de operações para Paste; `scripts/test_clipboard.sh` confirmou Cut/Paste real no filesystem sob Xvfb. A suíte, Clippy, auditoria, deny, cross-build Windows e smoke tests passaram nos lotes. A execução nativa Windows 10/11, acessibilidade e interoperabilidade com Explorer/CF_HDROP continuam pendentes; nenhuma dessas compatibilidades é declarada sem evidência. O sexto lote P1, Propriedades, foi implementado como diálogo somente leitura acionado pelo menu contextual. O handler reutiliza o snapshot `LoadedRow` já publicado, exige exatamente uma seleção, não segue links/reparse points e não relê ou altera o filesystem; o corpo usa Flickable para conter General, Security e Details. `scripts/test_properties.sh` abriu o diálogo para nome Unicode, exercitou a rolagem, confirmou que o arquivo permaneceu intacto e que o processo continuou ativo. A execução nativa Windows 10/11, ACLs reais, atributos completos, reparse points e escala de acessibilidade continuam pendentes. O sétimo lote P1, drag-and-drop, foi implementado com `DropArea`/`DataTransfer` para payloads internos e filtro `WinitWindowAccessor` para `HoveredFile`/`DroppedFile`. O hover valida arquivo absoluto regular, o drop usa `PathBuf` diretamente e despacha `Copy` pelo scheduler existente, com progresso, cancelamento e refresh; `scripts/test_*`, cross-build e manifesto passaram, enquanto o arraste efetivo do Explorer segue pendente de execução nativa. O próximo item isolado será busca recursiva, com engine cancelável e limites de filesystem antes de publicar resultados. O oitavo lote Search foi implementado: o filtro local permaneceu separado, o botão Buscar iniciou traversal recursivo em worker dedicado, resultados chegaram em batches com geração anti-stale, Cancelar interrompeu cooperativamente e refresh/navegação também cancelaram a busca. Testes unitários cobrem ordem determinística, case-insensitive, ocultos, limites, raiz relativa, symlink e coalescência; `scripts/test_recursive_search.sh` confirmou resultados em níveis profundos sob Xvfb. Indexação nativa do Windows, grandes árvores reais e execução nativa Windows 10/11 continuam pendentes. O próximo item isolado será thumbnails/preview, com parsers seguros, limites de memória/tempo e fallback para ícone genérico. O nono lote Preview foi implementado com `image` 0.25.10 em worker dedicado: decode por conteúdo para BMP/GIF/JPEG/PNG/WebP, rejeição de symlink/reparse point e arquivos não regulares, limites de 128 MiB de entrada, 8192×8192 de dimensão, 64 MiB de decode e thumbnail de 256px. O cache LRU é limitado a 128 entradas/32 MiB; gerações cancelam requests antigos, seleção múltipla/pastas/navegação escondem o painel e falhas usam fallback sem crash. `scripts/test_preview.sh` confirmou PNG válido, `.jpg` inválido e fechamento do painel; a matriz completa, Clippy, audit, deny, cross-check/build Windows GNU, manifesto e smoke tests passaram. O décimo lote Text Preview reutiliza o mesmo worker/cache e tenta apenas conteúdo literal: UTF-8, UTF-8 com BOM e UTF-16LE/BE com BOM, lendo no máximo 64 KiB, removendo BOM, rejeitando NUL/controles/binário e marcando truncamento. O Flickable escuro mostra Unicode sem links nem execução; testes unitários cobrem as duas endiannesses, BOM, binários, UTF-8 inválido, truncamento, arquivo vazio, erro de I/O e symlink; o smoke confirmou `nota.txt` no painel. PDF, vídeo, áudio, Office, handlers COM/Explorer e codecs externos continuam pendentes e não são ativados no processo principal. O décimo primeiro lote Settings foi implementado em `src/settings/`: o arquivo v1 fica em `%LOCALAPPDATA%\\Rovex` no Windows, sem privilégios, usa path hexadecimal nativo, parser limitado com fallback seguro e escrita temporária sincronizada; `MoveFileExW` aplica replace/write-through no alvo Windows. Startup restaura último diretório válido, Ocultos e coluna/direção de ordenação; os handlers salvam alterações imediatamente e o encerramento tenta salvar novamente. Testes unitários cobrem Unicode, replace sem temporário órfão, arquivo ausente/corrompido, limite de 16 KiB e chaves futuras; `scripts/test_settings.sh` alternou preferências e confirmou a segunda inicialização ativa. A execução nativa Windows 10/11, ACLs e redirecionamento de perfil permanecem pendentes. Preview de PDF, vídeo, áudio e Office continua fora do processo principal e requer workers isolados. O décimo segundo lote Recycle Bin foi implementado no alvo Windows com adapter `SHFileOperationW` UTF-16 e `FOF_ALLOWUNDO`, `FOF_NORECURSION`, confirmação única na UI, erro estruturado e nenhum fallback silencioso para exclusão permanente. A validação existente de raiz, symlink e diretórios não vazios foi preservada; o Linux mantém a exclusão permanente de desenvolvimento. O cross-check Windows GNU, testes host, Clippy e formatação passaram, mas Lixeira real, restauração pelo Explorer, ACLs, UNC/SMB, volumes removíveis e paths longos continuam pendentes de execução nativa. A API moderna `IFileOperation`/COM permanece evolução posterior. O décimo terceiro lote Distribution foi implementado em `scripts/package_windows_portable.sh` e `scripts/verify_windows_portable.sh`: a v0.1.11 publicou ZIP portable, manifesto e SHA-256, e a v0.1.12 consolidou o pipeline com documentação de assinatura ausente e checksum verificável. O lote seguinte Process Hardening centralizou `kill`/`wait`/join, manteve `Command::arg` sem shell, fechou stdin e criou testes fake de timeout/cancelamento; o smoke de JPEG XL em diretórios separados voltou a passar após corrigir coordenadas obsoletas e o uso incompatível de `-nostdin` no ffprobe. O próximo item isolado será validação nativa Windows 10/11 e evolução do adapter Shell para `IFileOperation`/COM, além de avaliar um instalador sem privilégios somente quando houver runner Windows e metadados de assinatura reais.

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

## Workflow de cada lote

Cada lote seguirá o ciclo: inspecionar o módulo existente; definir risco e critério de aceite; implementar uma mudança pequena; executar `cargo fmt`, `cargo check`, testes focados e validações de plataforma pertinentes; revisar segurança, concorrência, desempenho, UX e acessibilidade; atualizar documentação; criar commit descritivo; e somente então avançar.

## Critérios de pronto

Uma feature só será considerada pronta quando houver implementação real, fluxo de erro estruturado, cancelamento quando aplicável, recuperação ou comportamento seguro, testes relevantes, documentação atualizada, validação Windows 10/11 quando aplicável e evidência de que resultados obsoletos não substituem o estado atual.
