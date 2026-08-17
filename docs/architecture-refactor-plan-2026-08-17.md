# Plano de refatoração arquitetural do Rovex

**Data:** 2026-08-17  
**Versão de partida:** v0.1.8 / commit `92f46b3`  
**Objetivo:** melhorar coesão, legibilidade, encapsulamento e manutenção sem mudar o comportamento funcional.

## Linha de base

O crate continua sendo um único pacote com uma biblioteca (`rovex_core`) e um binário (`rovex`). A árvore de produção possui cinco módulos declarados em `src/lib.rs`: `converters.rs`, `desktop.rs`, `filesystem.rs`, `operations.rs` e `security.rs`. A auditoria inicial encontrou dois centros de responsabilidade já coesos (`filesystem`, `operations` e `security`) e dois módulos maiores (`desktop.rs` com 2.566 linhas e `converters.rs` com 1.438 linhas).

A validação antes da refatoração passou com `cargo fmt --check`, `cargo check`, `cargo test` e `cargo clippy --all-targets --all-features -- -D warnings`. Foram registrados 43 testes aprovados e 2 ignorados explicitamente. O checkpoint de rollback está publicado na branch `backup/before-architecture-refactor-2026-08-17`.

## Diagnóstico

`desktop.rs` mistura quatro responsabilidades diferentes: estado de apresentação (linhas, seleção, filtro, abas), descoberta de locais conhecidos, execução assíncrona de tarefas e composição/callbacks da janela Slint. Essas responsabilidades se comunicam por APIs internas identificáveis, portanto a divisão pode reduzir acoplamento sem criar uma camada artificial.

`converters.rs` é grande, mas o domínio é coeso e suas seções internas já têm uma direção clara: tipos e política de conversão, resolução de backends, execução de processos e orquestração. Ele depende da API interna de publicação atômica em `operations` e das validações em `security`. Não será dividido nesta etapa apenas por tamanho.

`filesystem.rs` concentra leitura e classificação de entradas; `operations.rs` concentra mutações seguras de arquivos; `security.rs` concentra invariantes de origem/destino. Esses módulos devem permanecer inteiros, salvo uma necessidade concreta descoberta durante a implementação.

## Estrutura-alvo proporcional

```text
src/
├── lib.rs                 # fachada do crate e reexports de domínio
├── main.rs                # seleção CLI/UI
├── filesystem.rs          # leitura/classificação de diretórios
├── security.rs            # validação e políticas de segurança
├── operations.rs          # cópia/movimentação/renomeação/exclusão seguras
├── converters.rs          # domínio completo de conversão e descoberta FFmpeg
├── desktop.rs             # fachada Slint e composição da aplicação
└── desktop/
    ├── locations.rs       # locais iniciais e known folders por plataforma
    ├── state.rs           # estado de abas/seleção/listagem/filtro e view-model
    └── jobs.rs             # tarefas, executores, filas e schedulers assíncronos
```

A convenção `desktop.rs` mais `desktop/` é deliberada: `desktop.rs` permanece como fachada e os submódulos são carregados a partir da pasta do mesmo módulo. Não será criado um `mod.rs`, nem serão criados arquivos por função ou por tipo.

### `desktop::locations`

Este módulo isolará somente a descoberta de locais padrão: `LocationEntry`, `user_home`, known folders do Windows e a lista inicial multiplataforma. A API interna será uma função pequena que recebe o caminho inicial e devolve locais existentes sem duplicação. O código Win32 permanecerá protegido por `cfg(windows)`.

### `desktop::state`

Este módulo conterá tipos e regras puras do estado da apresentação: `LoadedRow`, `LoadedDirectory`, `SelectionState`, `NavigationHistory`, `TabManager`, `row_icon`, `row_from_entry`, filtragem, mensagens de estado, formatação de tamanho e carregamento de diretório. Ele poderá conhecer `filesystem` e os tipos Slint gerados apenas quando isso for necessário para atualizar linhas de aba; não conhecerá schedulers ou callbacks.

### `desktop::jobs`

Este módulo conterá a coordenação assíncrona das tarefas: requests/outcomes de operações e conversões, execução de uma operação, execução de uma conversão, progresso, cancelamento e os schedulers de carga, filtro, operações e conversões. A API permanecerá restrita ao crate. `jobs` dependerá de `state` para carregar diretórios e de `operations`/`converters` para executar o domínio; `state` não dependerá de `jobs`.

### `desktop.rs`

A fachada manterá `slint::include_modules!()`, os adaptadores mínimos entre view-model e `MainWindow`, os diálogos e o registro de callbacks. Ela será o composition root: monta modelos, cria schedulers, conecta callbacks e inicia a janela. A wiring da UI permanecerá junta propositalmente, porque separar cada grupo de callbacks em arquivos independentes aumentaria a quantidade de tipos compartilhados e esconderia o fluxo de inicialização.

## Regras de encapsulamento

Os novos submódulos serão privados (`mod locations;`, `mod state;`, `mod jobs;`). Tipos e funções só serão `pub(crate)` se precisarem ser usados por `desktop.rs` ou por outro submódulo. Não será ampliada a API pública do crate. A fachada pública existente (`desktop::run` e os reexports de domínio em `lib.rs`) será preservada para evitar mudança no binário e em consumidores internos.

Nenhuma trait, factory, manager adicional, wrapper ou crate novo será criado. `TabManager`, `LoadScheduler` e os schedulers atuais continuarão sendo as estruturas concretas responsáveis pelas suas regras, apenas em fronteiras de módulo mais claras.

## Estratégia incremental

A migração será feita em três movimentos verificáveis: primeiro `locations` e `state`, depois `jobs`, e por fim a limpeza de imports e visibilidade da fachada. Após cada movimento serão executados `cargo fmt --check`, `cargo check` e `cargo test`; Clippy será executado ao final de cada grupo. O comportamento da UI, conversores, operações, threads, cancelamento e erros será tratado como contrato.

A revisão final comparará número e tamanho dos arquivos, dependências entre módulos, itens públicos, duplicações e testes. Se uma extração aumentar a complexidade ou exigir visibilidade excessiva, ela será revertida ou simplificada.

## Fontes de orientação

[1]: https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html "The Rust Programming Language — Control Scope and Privacy with Modules"
[2]: https://doc.rust-lang.org/cargo/guide/project-layout.html "The Cargo Book — Package Layout"
[3]: https://rust-lang.github.io/api-guidelines/about.html "Rust API Guidelines"
[4]: https://github.com/BurntSushi/ripgrep "BurntSushi/ripgrep"
[5]: https://github.com/sharkdp/fd/tree/master/src "sharkdp/fd — src"

As fontes [1] e [2] sustentam módulos por responsabilidade e convenções de layout; [3] orienta APIs pequenas, previsíveis e encapsuladas; [4] e [5] servem como referências reais de projetos Rust que separam domínios sem exigir um arquivo por função.
