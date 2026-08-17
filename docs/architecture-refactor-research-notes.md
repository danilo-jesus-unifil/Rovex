# Notas de pesquisa — refatoração arquitetural

## Fontes oficiais consultadas

### Rust Book — módulos, escopo e privacidade
URL: https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html

A documentação oficial descreve módulos como uma forma de organizar código por legibilidade e reutilização, além de controlar a privacidade dos itens. Itens são privados por padrão; a visibilidade pública deve ser escolhida quando existe uma necessidade real de expor uma API. A documentação também enfatiza que organizar definições em grupos nomeados ajuda desenvolvedores a encontrar responsabilidades e saber onde adicionar funcionalidades.

Aplicação ao Rovex: usar módulos por responsabilidade/domínio, preservar a privacidade por padrão, limitar `pub`/`pub(crate)` ao necessário e evitar separar funções apenas por contagem de linhas.

### Rust API Guidelines — propósito e escopo
URL: https://rust-lang.github.io/api-guidelines/about.html

As API Guidelines são recomendações baseadas na experiência da equipe de bibliotecas do Rust e do ecossistema. O texto esclarece que são diretrizes, não mandatos rígidos, e devem ser usadas como considerações para APIs idiomáticas e interoperáveis. A página organiza as recomendações em temas como nomenclatura, previsibilidade, flexibilidade, segurança de tipos, dependabilidade, depuração e evolução futura.

Aplicação ao Rovex: cada novo módulo deve oferecer uma API interna pequena, previsível e coerente; a refatoração não deve criar abstrações artificiais, traits sem necessidade ou tornar detalhes de implementação públicos apenas para resolver imports.

## Implicações preliminares

1. A fronteira mais natural é entre domínio de filesystem/segurança/operações, domínio de conversão e camada de apresentação/coordenação desktop.
2. `filesystem.rs`, `operations.rs` e `security.rs` já mostram boa coesão e não devem ser fragmentados sem ganho arquitetural concreto.
3. `desktop.rs` concentra estado de UI, modelo de linhas, abas, seleção, filtragem, carregadores, schedulers, operações e callbacks; é o principal candidato a uma divisão por responsabilidades, mas precisa ser dividido gradualmente para evitar dependências circulares.
4. `converters.rs` é grande, porém contém fronteiras internas claras entre tipos de conversão, descoberta de backends, execução de processos, orquestração e testes. A divisão só será feita se reduzir acoplamento e mantiver uma API de conversão pequena.
5. A validação deve acompanhar cada etapa: format/check, testes, clippy e execução real, seguida de uma auditoria contra fragmentação e aumento indevido de visibilidade.

## Convenções e exemplo real

### Cargo Book — layout de pacote
URL: https://doc.rust-lang.org/cargo/guide/project-layout.html

O Cargo usa convenções de localização para facilitar que alguém novo entenda um pacote. O código de produção fica em `src`, com `lib.rs` como raiz de biblioteca e `main.rs` como binário padrão; subdiretórios e módulos devem ser usados quando um alvo realmente precisa de múltiplos arquivos. A convenção é um ponto de partida, não uma exigência de um arquivo por função.

Aplicação ao Rovex: manter `lib.rs` como fachada de módulos do crate e usar arquivos/subdiretórios apenas para domínios coerentes. Não há justificativa para transformar o projeto em workspace ou múltiplos crates nesta etapa.

### ripgrep — projeto Rust real
URL: https://github.com/BurntSushi/ripgrep

A árvore observada separa responsabilidades de projeto em diretórios como `crates`, `tests`, `ci`, `pkg`, `.github` e `scripts`, mantendo a organização de runtime distribuída por crates/domínios em vez de uma coleção de arquivos genéricos. O projeto também mantém documentação, testes, empacotamento e CI como áreas reconhecíveis.

Aplicação ao Rovex: a referência reforça separar domínio e infraestrutura por fronteiras reais, mas não copiar a escala de ripgrep. Para o Rovex, a solução proporcional é manter um único crate e extrair apenas partes coesas de `desktop.rs`; `filesystem.rs`, `operations.rs`, `security.rs` e provavelmente `converters.rs` devem permanecer como âncoras de domínio, com submódulos somente onde houver API interna clara.

### fd — projeto Rust real menor
URL: https://github.com/sharkdp/fd/tree/master/src

O diretório `src` do fd mantém `main.rs` e arquivos de domínio (`filesystem.rs`, `filetypes.rs`, `dir_entry.rs`, `walk.rs`, `output.rs`, `error.rs`, entre outros), mas cria subdiretórios apenas para agrupamentos que possuem coesão própria (`exec`, `filter`, `fmt`). Isso é uma referência útil para o Rovex: arquivos relativamente grandes podem permanecer quando o domínio é coeso, enquanto grupos de funcionalidades com regras e testes próprios podem virar submódulos.

A estrutura também mostra que organização por responsabilidade não exige uma camada genérica para cada categoria sintática. O critério deve ser navegabilidade e isolamento real, não o tamanho isolado de um arquivo.
