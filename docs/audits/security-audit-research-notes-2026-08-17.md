# Notas de pesquisa: auditoria de segurança

## OWASP Top 10:2025

URL: https://owasp.org/Top10/2025/

A página oficial identifica o OWASP Top 10:2025 como um documento de conscientização para desenvolvedores e segurança de aplicações web, baseado em consenso amplo sobre riscos críticos. A lista atual inclui: A01 Broken Access Control; A02 Security Misconfiguration; A03 Software Supply Chain Failures; A04 Cryptographic Failures; A05 Injection; A06 Insecure Design; A07 Authentication Failures; A08 Software or Data Integrity Failures; A09 Security Logging and Alerting Failures; e A10 Mishandling of Exceptional Conditions.

Aplicação ao Rovex: o Top 10 não é um checklist específico de desktop e não deve ser aplicado mecanicamente. Para este aplicativo local, os temas mais relevantes são controle de acesso aos caminhos/arquivos, configuração segura, cadeia de suprimentos, integridade do build/artefatos, injeção em execução de processos, desenho seguro e tratamento correto de condições excepcionais. Autenticação, logging remoto e riscos web não são assumidos como presentes sem evidência no código.

## ANSSI Secure Rust Guidelines

URL: https://anssi-fr.github.io/rust-guide/

A guia é apresentada como uma referência ainda marcada como instável para desenvolvimento de aplicações com requisitos fortes de segurança. Ela organiza recomendações por ambiente de desenvolvimento, bibliotecas, nomenclatura, operações inteiras, tratamento de erros, garantias da linguagem, Rust inseguro, FFI e biblioteca padrão. A introdução destaca que Rust oferece gerenciamento automático de memória e prevenção de data races, mas que construções da linguagem, ferramentas de build e bibliotecas ainda podem introduzir riscos quando usadas de forma inadequada ou pouco clara.

Aplicação ao Rovex: revisar `unsafe` de FFI Windows com contratos explícitos; manter `unwrap`/`expect` restritos aos testes; preservar erros estruturados nas entradas e processos externos; auditar crates e toolchain; e tratar a revisão de segurança como um processo contínuo, não como prova absoluta de ausência de bugs.

## OWASP A03, A05 e A10; RustSec

Fontes: https://owasp.org/Top10/2025/A03_2025-Software_Supply_Chain_Failures/ ; https://owasp.org/Top10/2025/A05_2025-Injection/ ; https://owasp.org/Top10/2025/A10_2025-Mishandling_of_Exceptional_Conditions/ ; https://rustsec.org/

A03:2025 trata falhas de cadeia de suprimentos como problemas de construção, distribuição e atualização, incluindo componentes transitivos, ferramentas de CI/CD, origem confiável, inventário, SBOM, monitoramento e integridade de artefatos. A05:2025 define injection como entrada não confiável que chega a um interpretador e recomenda manter dados separados de comandos, preferindo APIs seguras e argumentos parametrizados. A10:2025 cobre tratamento incorreto de condições excepcionais, incluindo falhas de validação, condições de privilégio, falhas abertas, limpeza incompleta, consumo ilimitado de recursos e estados parcialmente concluídos; recomenda tratar erros no ponto de ocorrência, falhar fechado, liberar recursos e impor limites.

A página RustSec descreve `cargo-audit` como ferramenta para auditar `Cargo.lock` contra vulnerabilidades do ecossistema Rust, `cargo-deny` como auditoria de vulnerabilidades, bans, licenças, fontes e múltiplas versões, e `cargo-auditable` como forma de embutir a árvore de dependências nos binários.

Aplicação ao Rovex: os comandos `Command::new(absolute_or_resolved_path).arg(...)` mantêm dados de caminhos separados da estrutura do comando e não usam shell; a auditoria ainda precisa testar resolução de backend e ambiente. O projeto já usa `cargo audit`/`cargo deny`, mas não gera SBOM/proveniência assinada. Os limites existentes de conversão e cancelamento devem ser testados contra arquivos inválidos, processos que não terminam e saídas parciais.

## Achados reproduzidos no estado atual

Os smokes `capture_gui.sh`, `capture_context_menu.sh` e `capture_conversion_menu.sh` executaram no binário release. A captura de tema mostra a interface escura consistente no caminho principal, sem cortes aparentes. A captura do menu contextual reproduz uma divergência de produto: o texto informa que conversões usam “FFmpeg instalado no PATH”, embora o resolvedor real também use override absoluto, PATH persistente/registro, App Paths, SearchPathW, diretório do executável, variáveis `ROVEX_*` e diretórios conhecidos. Esse texto é enganoso e deve ser corrigido.

A captura também confirma que o menu contextual e a toolbar visualizam corretamente ações de cópia, movimentação, renomeação, exclusão e conversões desabilitadas para uma pasta, com a ação de exclusão destacada em perigo. Nenhuma falha de composição visual foi reproduzida nesse fluxo.

## Diálogo de conversão reproduzido

A captura `artifacts/rovex-jxl-confirm.png` mostra que o diálogo de confirmação funciona e é legível, mas usa um painel fixo alto de 560×330 px para uma mensagem curta. Há uma área vazia grande entre a explicação e os botões, e o texto de confirmação não informa o caminho exato da saída nem o backend resolvido. Isso é uma oportunidade de UX/manutenção com evidência visual, não uma falha de segurança; qualquer ajuste deve preservar o fluxo de confirmação e o teste de conversão real.

## Validação visual pós-correção

Após recompilar, os smokes de janela, menu contextual, conversão e abas passaram. A nova microcópia do menu contextual está correta em conteúdo, porém é truncada visualmente pelo `overflow: elide` porque a largura do menu é limitada; ainda comunica `PATH`, `Windows`, app e `ROVEX_*...`, mas pode ser encurtada para uma frase que caiba integralmente. O diálogo de conversão está centralizado com a altura reduzida, sem área vazia tão grande, e o botão Confirmar foi reposicionado no smoke para a coordenada real y=464. A conversão JPEG XL e o cenário de diretórios separados passaram após a atualização dos scripts.

## Correção da validação do artefato visual

A captura `rovex-conversion-menu.png` anterior pertence ao smoke de menu de conversão e não foi sobrescrita pelo `capture_context_menu.sh`; por isso ainda mostrava a string anterior. O artefato correto, `rovex-context-menu.png`, confirma que `Backends locais: PATH, Windows ou override` cabe integralmente no painel e que a correção visual foi efetivamente compilada no release.

## Inspeção visual final

A captura final `rovex-context-menu.png` confirma que a microcópia encurtada cabe integralmente como `Backends locais: PATH, Windows ou override`, sem alterar a hierarquia das ações. A captura `rovex-tabs-two.png` confirma duas abas visíveis, a aba ativa destacada, botão de nova aba separado e listagem estável no segundo contexto. O fluxo de abrir, alternar e fechar abas passou novamente.
