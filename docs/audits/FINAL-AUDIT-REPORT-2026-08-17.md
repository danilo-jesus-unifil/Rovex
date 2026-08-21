# Rovex: Relatório final de auditoria e hardening

**Data:** 17 de agosto de 2026
**Versão auditada:** 0.1.8
**Escopo:** código Rust/Slint, filesystem, operações, conversores FFmpeg/ffprobe, concorrência, UI, documentação, dependências, builds e execução real.

## Resultado da auditoria

Os fluxos de filesystem, conversão JPEG XL, menu contextual, diálogo de confirmação, abas, smoke gráfico e casos extremos do CLI foram reproduzidos no estado real do repositório. Os problemas encontrados no escopo exercitado foram corrigidos e validados novamente com compilação, testes e execução.

> **Resultado:** não foi encontrada vulnerabilidade explorável no escopo auditado. O código de produção usa blocos `unsafe` somente nas integrações FFI Win32 de resolução de ambiente e Known Folders, com contratos de ponteiros e buffers documentados imediatamente nos call sites. Não há `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `unwrap()` ou `expect()` em caminhos de produção após a separação dos módulos de teste. A execução externa usa `Command` com argumentos separados e não monta comandos por shell.

Isso não equivale a uma certificação formal de segurança nem a uma validação nativa completa do Windows. O build Windows GNU foi confirmado por cross-build, mas ainda é necessário executar o binário em Windows 10/11 real para validar DPI, acessibilidade, permissões, junctions/reparse points, paths longos, UNC/SMB, arquivos em uso e empacotamento.

## Achados reproduzidos e correções

| Área | Evidência reproduzida | Risco | Correção aplicada | Regressão |
|---|---|---:|---|---|
| Captura de processos | FFmpeg era acompanhado enquanto stderr só era lido depois do processo terminar. Um backend com stderr suficientemente grande poderia bloquear no pipe antes de terminar. | Médio | stdout/stderr agora são drenados por leitores paralelos nomeados; a memória retida é limitada a 64 KiB por pipe e o processo é encerrado/aguardado se a criação de um leitor falhar. | Teste de limite de diagnóstico; conversões reais com FFmpeg/ffprobe. |
| Saída de diagnóstico | A captura anterior acumulava a saída inteira de um processo externo. | Médio | `read_limited_output` retém no máximo 64 KiB, continua drenando o pipe e retorna erro estruturado quando o limite é excedido. | `diagnostico_de_backend_tem_limite_de_tamanho`. |
| Caminhos de erro dos schedulers | `unreachable!` era usado se o canal de um scheduler fosse fechado em um estado não esperado. | Baixo | `OperationRequest` passou a ser clonável; falhas de envio retornam o request original e limpam o estado `busy`, sem caminho de panic. | `cargo check`, testes e Clippy estrito. |
| Microcópia da UI | O menu contextual dizia que conversões usavam somente “FFmpeg instalado no PATH”, embora o resolvedor suporte múltiplas camadas locais. | Baixo | Texto corrigido para `Backends locais: PATH, Windows ou override`, validado sem truncamento na captura final. | `capture_context_menu.sh`. |
| Densidade do diálogo | O diálogo de confirmação usava 330 px para uma mensagem curta e deixava uma área vazia excessiva. | Baixo | Altura reduzida para 280 px, fórmula de centralização atualizada e coordenadas dos smokes corrigidas após uma falha real de regressão. | JPEG XL pela UI e cenário com binário/imagem em diretórios distintos. |
| Documentação canônica | Relatórios gerais afirmavam que abas, conversores e operações de UI não existiam e registravam contagens antigas de testes/MSRV. | Médio para manutenção | `../reports/IMPLEMENTATION_REPORT.md`, `SECURITY.md`, `../reference/known-issues.md`, `../reference/testing.md`, `COMPATIBILITY.md`, `../reference/DEPENDENCIES.md`, `../reports/FINAL_STABILITY_REPORT.md` e `../releases/RELEASE_NOTES.md` foram reconciliados ou marcados como históricos. | Busca pós-correção por afirmações obsoletas e `git diff --check`. |

## Validação de filesystem e CLI

O novo script `scripts/audit_edge_cases.sh` executou o binário release contra diretórios vazios, caminhos com espaços, Unicode, caracteres especiais, nomes longos, arquivo usado como diretório, caminho inexistente e diretório sem permissão. Todos os casos retornaram códigos e mensagens coerentes. Não foram observados crashes, falsos sucessos ou perda de dados.

A listagem continua usando `symlink_metadata`, classifica links sem seguir o destino para a apresentação e ordena diretórios antes de arquivos. A validação de destinos mantém recusa de caminhos relativos, raiz, componentes pai simbólicos, destinos existentes e colisões equivalentes. A cópia publica o destino somente depois de validação e escrita completa.

## Validação dos conversores

As conversões reais foram exercitadas com FFmpeg e ffprobe instalados no ambiente. O teste unitário ignorado explicitamente foi executado manualmente e passou para JPEG XL, PNG, Opus e FLAC, incluindo recusa de saída já existente. O smoke de JPEG XL pela UI criou e verificou um arquivo de saída real de 67 bytes. O teste com executável copiado para uma pasta e imagem em outra confirmou a saída `entrada.jxl` e o codec `jpegxl` validado por ffprobe.

A resolução de backend permanece sem download em runtime e usa candidatos locais, overrides absolutos, PATH, mecanismos Windows, diretório do executável, diretório de trabalho, variáveis de ambiente e instalações locais conhecidas. A execução mantém dados de caminho separados da estrutura do comando; essa é a prática recomendada para evitar que entrada de caminho seja interpretada como estrutura de comando [1].

## Validação da UI e concorrência

Os smokes gráficos confirmaram inicialização do processo sob Xvfb, tema escuro, toolbar, lista, menu contextual, estados de ação, diálogo de conversão e abas. O fluxo de abas abriu uma segunda aba, alternou para a primeira e fechou a segunda. A captura final mostra a aba ativa destacada, botão de nova aba separado e listagem estável.

O loader, filtro, operações e conversões continuam separados em workers nomeados e devolvem atualizações ao event loop do Slint. Filtragem e carregamento descartam resultados obsoletos por geração. O hardening de pipes evita que um backend externo que escreva muito stderr impeça a conclusão do processo, enquanto timeout, cancelamento e cleanup continuam ativos.

## Dependências e supply chain

A bateria passou em `cargo audit` com código 0 e em `cargo deny check` para advisories, bans, licenças e fontes. Permanecem quatro warnings de manutenção transitivos da cadeia Slint: `bincode`, `paste`, `rustybuzz` e `ttf-parser`: sem advisory explorável ou atualização segura indicada na resolução verificada. Eles continuam visíveis e não foram mascarados por exceções.

A auditoria não encontrou segredo versionado, download de executável em runtime ou uso de shell para conversão. A documentação, porém, ainda não implementa SBOM ou assinatura/proveniência de artefatos. Isso permanece como melhoria de distribuição, não como vulnerabilidade corrigida nesta rodada. O OWASP classifica inventário de componentes, dependências transitivas, integridade de artefatos e segurança do CI/CD como parte de falhas de cadeia de suprimentos [2].

## Resultado dos gates técnicos

| Gate | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Passou. |
| `cargo check --all-targets --all-features` | Passou. |
| `cargo test --all-targets --all-features` | 44 passaram, 0 falharam, 2 ignorados explicitamente. |
| Testes ignorados executados manualmente | 2 passaram: benchmark de filtro e conversões reais. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passou sem warnings. |
| `cargo audit` | Passou; sem vulnerabilidade explorável, com quatro warnings de manutenção transitivos documentados. |
| `cargo deny check` | Passou em advisories, bans, licenças e fontes. |
| `cargo check --target x86_64-pc-windows-gnu` | Passou. |
| `cargo build --release` | Passou. |
| `cargo build --release --target x86_64-pc-windows-gnu` | Passou. |
| Casos extremos do CLI | Passaram. |
| Smokes gráficos | Passaram: janela, tema, menu contextual, conversão e abas. |
| Conversão JPEG XL pela UI | Passou com arquivo real publicado. |
| JPEG XL com diretórios separados | Passou com codec validado por ffprobe. |
| `git diff --check` | Passou. |

## Limitações remanescentes

A auditoria não implementou funcionalidades novas que não eram necessárias para corrigir um defeito comprovado. Permanecem fora do escopo atual pesquisa global, thumbnails, pré-visualização, drag and drop, integração com shell, OCR, instalador, assinatura, atualização automática e acesso à Lixeira. A execução nativa Windows 10/11, DPI, acessibilidade nativa, caminhos UNC/SMB, reparse points, paths longos e instalador continuam gates de distribuição.

O issue #1 do GitHub está parcialmente atendido: abas reais, navegação e identificação semântica de extensões estão implementadas; a Lixeira e uma biblioteca visual específica de ícones por linguagem ainda são trabalho futuro. Esses itens não foram acoplados artificialmente à auditoria porque não representam falhas comprovadas nos fluxos atuais.

## Referências

[1]: https://doc.rust-lang.org/std/process/struct.Command.html "Rust Standard Library: std::process::Command"

[2]: https://owasp.org/Top10/2025/A03_2025-Software_Supply_Chain_Failures/ "OWASP Top 10:2025: A03 Software Supply Chain Failures"

[3]: https://owasp.org/Top10/2025/A05_2025-Injection/ "OWASP Top 10:2025: A05 Injection"

[4]: https://owasp.org/Top10/2025/A10_2025-Mishandling_of_Exceptional_Conditions/ "OWASP Top 10:2025: A10 Mishandling of Exceptional Conditions"

[5]: https://rustsec.org/ "RustSec Advisory Database"

[6]: https://anssi-fr.github.io/rust-guide/ "ANSSI: Secure Rust Guidelines"
