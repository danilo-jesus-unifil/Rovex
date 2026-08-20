# Relatório de release — Rovex v0.1.24

**Data:** 2026-08-20

A v0.1.24 é uma release de endurecimento da descoberta de conversores e de manutenção arquitetural. O follow-up da auditoria confirmou que retirar somente a inserção explícita do diretório de trabalho não bastava: `SearchPathW` com `lpPath = NULL` e `where.exe` ainda podiam consultar o CWD em uma resolução de FFmpeg/ffprobe. A release remove esses dois caminhos ambíguos, preserva as fontes explícitas e adiciona contratos para impedir regressão.

| Item | Resultado |
|---|---|
| Versão | `0.1.24` |
| Branch de backup | `backup/before-ffmpeg-cwd-hardening-20260820` |
| Código | `src/converters/backend.rs` e `src/converters/windows_backend.rs` |
| Testes | `descoberta_nao_adiciona_cwd_implicitamente` e cobertura do diretório adjacente explícito |
| Gate | `scripts/test_ffmpeg_discovery_contract.sh` |
| Pesquisa | `docs/research/ffmpeg-discovery-research.md` e `artifacts/validation/next-cycle-executable-search-research-2026-08-20.md` |

## Falha confirmada

A implementação enumerava candidatos absolutos diretamente, mas também inseria o diretório atual como fallback. No Windows, `SearchPathW` foi chamado com `lpPath = NULL`, cuja ordem depende de `SafeProcessSearchMode`; a documentação oficial informa que o CWD pode ser pesquisado antes do PATH quando o valor é zero, que é o padrão documentado.[1] O comando `where` também pesquisa por padrão o diretório atual e o PATH.[2]

Assim, mesmo que a conversão exigisse uma ação explícita e passasse um caminho absoluto para `Command`, um backend plantado no CWD poderia ser selecionado quando as fontes anteriores falhassem ou através dos próprios fallbacks nativos. Uma discussão do ecossistema Rust registra o mesmo risco para nomes relativos no Windows e cita um caso de execução arbitrária corrigido em ferramenta Rust; a recomendação técnica é resolver e validar um caminho absoluto antes do spawn.[3]

> O problema foi confirmado pela combinação do código atual com os contratos documentados das APIs, não por uma suposição sobre FFmpeg. A correção foi acompanhada por teste e gate estrutural; a execução nativa Windows continua necessária para validar todos os detalhes do ambiente real.

## Correção

`backend_candidates` deixou de inserir o CWD implicitamente. O adapter Windows deixou de usar `SearchPathW(lpPath = NULL)` e `where.exe`, eliminando as duas fontes que podiam consultar o CWD fora da política explícita do Rovex.

A cadeia restante preserva overrides absolutos, PATH herdado, PATH persistente do usuário e do sistema, App Paths, diretório do executável, diretório adjacente explicitamente informado para localizar o par FFmpeg/ffprobe, raízes conhecidas, WinGet, Chocolatey e Scoop com limites. `is_backend_file` continua exigindo caminho absoluto e arquivo regular antes da execução. A remoção não altera argumentos, não usa shell, não baixa executáveis e não autentica por assinatura ou hash os candidatos que o usuário autorizou por PATH/Registro.

Como manutenção adicional, os testes que estavam inline em `src/security.rs` foram movidos para `src/security/tests.rs`, reduzindo o módulo principal de 467 para 312 linhas; nenhum arquivo Rust do diretório `src` ultrapassa 400 linhas.

## Validação incrementada

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| Testes focados de conversores | 11 aprovados; 1 ignorado explicitamente |
| Suíte host no ciclo | 106 aprovados; 2 ignorados explicitamente; 0 falhas |
| Clippy host | Aprovado com `-D warnings` |
| Check/Clippy Windows GNU | Aprovados com `-D warnings` |
| Contrato FFmpeg | CWD implícito, SearchPathW e where.exe bloqueados; caminho absoluto exigido |
| Limite modular | Maior arquivo Rust: 370 linhas |
| Gates anteriores | Contratos de ativação, Windows nativo e nomes reservados preservados |

O teste host evita mutar PATH global: se o processo já declara o CWD explicitamente no PATH, o caso não transforma uma configuração deliberada do usuário em falha; caso contrário, prova que a função não adiciona o CWD e que o diretório adjacente passado explicitamente continua presente. A compilação cross-Windows valida as ramificações condicionais, e o job `windows-latest` deve confirmar a suíte e o smoke nativos.

## Limitações honestas

A remoção de `SearchPathW` e `where.exe` reduz a ambiguidade, mas não autentica executáveis encontrados em PATH, Registro, WinGet, Chocolatey, Scoop ou diretórios graváveis pelo usuário. Uma futura política poderá exigir assinatura, hash ou diretórios confiáveis, mas isso precisa considerar atualizações legítimas e instalação portable.

Continuam pendentes testes nativos específicos de DLLs carregadas pelo FFmpeg, TOCTOU entre validação e spawn, Job Objects para descendentes, ACLs, arquivos bloqueados, UNC/SMB, namespaces extended-length, volumes removíveis, disco cheio, DPI, acessibilidade e execução gráfica interativa completa em Windows 10/11. A distribuição segue portable, sem assinatura digital, instalador MSI/MSIX ou atualização automática.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-searchpathw "SearchPathW function"
[2]: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/where "where command"
[3]: https://internals.rust-lang.org/t/std-command-resolve-to-avoid-security-issues-on-windows/14800 "std::process::Command resolve() to avoid security issues on Windows?"
