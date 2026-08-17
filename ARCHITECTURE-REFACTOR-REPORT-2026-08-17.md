# Relatório de refatoração arquitetural do Rovex

**Data:** 2026-08-17  
**Versão de partida:** v0.1.8  
**Commits da refatoração:** `6c11eba`, `aa70ec4`  
**Branch de rollback:** `backup/before-architecture-refactor-2026-08-17`

## Resultado executivo

A refatoração foi concluída como uma mudança estrutural, sem alterar os contratos funcionais observados. O projeto continua sendo um único crate Rust com biblioteca `rovex_core` e binário `rovex`. A principal correção foi retirar de `src/desktop.rs` as responsabilidades que não pertencem ao composition root da interface: estado/view-model, descoberta de locais e coordenação de jobs assíncronos.

A arquitetura resultante é menor em acoplamento e mais navegável, mas não foi fragmentada artificialmente. `filesystem.rs`, `operations.rs`, `security.rs` e `converters.rs` permaneceram como módulos de domínio coesos. Não foram criados traits, factories, wrappers, crates adicionais ou arquivos por função.

## Comparação antes/depois

| Área | Antes | Depois | Motivo da decisão |
|---|---|---|---|
| Fachada desktop | `src/desktop.rs` com 2.566 linhas reunia UI, estado, locais, jobs e callbacks | `src/desktop.rs` com 911 linhas mantém composição Slint, diálogos e callbacks | A fachada continua sendo o composition root e não espalha a wiring da UI por arquivos pequenos. |
| Estado da interface | Misturado em `desktop.rs` | `src/desktop/state.rs` com 663 linhas | Reúne seleção, abas, filtragem, view-model, carregamento de diretório e regras puras relacionadas. |
| Jobs assíncronos | Misturados com callbacks e estado | `src/desktop/jobs.rs` com 892 linhas | Reúne requests, outcomes, progresso, cancelamento e schedulers que compartilham ciclo de vida e dependências. |
| Locais iniciais | Misturados com a inicialização da janela | `src/desktop/locations.rs` com 132 linhas | Isola known folders do Windows e fallback multiplataforma, incluindo `cfg(windows)`. |
| Conversão | `src/converters.rs` com 1.438 linhas | Permaneceu unido | O domínio já tinha coesão interna entre descoberta, execução e orquestração; dividir por tamanho aumentaria a superfície entre módulos. |
| Filesystem, operações e segurança | Módulos independentes e coesos | Permaneceram independentes | Já representavam fronteiras naturais e não exigiam redistribuição. |

A árvore de produção final é:

```text
src/
├── lib.rs
├── main.rs
├── filesystem.rs
├── security.rs
├── operations.rs
├── converters.rs
├── desktop.rs
└── desktop/
    ├── locations.rs
    ├── state.rs
    └── jobs.rs
```

## Fronteiras e encapsulamento

Os submódulos `desktop::locations`, `desktop::state` e `desktop::jobs` são privados à árvore desktop. A comunicação necessária com a fachada usa `pub(super)`, sem ampliar a API pública do crate. Os campos internos de seleção, histórico, schedulers e conversão continuam privados; apenas dados efetivamente consumidos por um módulo irmão ou pela fachada atravessam a fronteira.

`state` não depende de `jobs`. O módulo de jobs depende do view-model de estado para carregar e atualizar listagens, além de depender dos domínios existentes de operações e conversão. A fachada desktop monta os componentes e conecta callbacks. Essa direção reduz a mistura entre regras de estado e execução assíncrona sem criar uma camada de abstração que apenas repassaria chamadas.

A convenção usada foi `desktop.rs` como arquivo-fachada com a pasta `desktop/` para submódulos. Não foi criado `mod.rs`, alinhando a organização à convenção moderna de arquivo de módulo mais diretório de submódulos documentada pelo ecossistema Rust.

## Pesquisa e decisões evitadas

A documentação oficial do Rust descreve módulos como mecanismos para organizar código por legibilidade e controlar privacidade, com itens privados por padrão [1]. O Cargo recomenda convenções de layout que facilitem a navegação em um pacote novo, sem exigir um arquivo para cada função [2]. As Rust API Guidelines tratam desenho de APIs, previsibilidade, dependabilidade e evolução como recomendações a serem aplicadas com julgamento, não como uma regra rígida [3].

As árvores de projetos reais `ripgrep` e `fd` foram consultadas. `ripgrep` separa áreas de domínio, testes, CI e empacotamento em uma escala muito maior que a do Rovex [4]. `fd` mantém arquivos de domínio em `src` e usa subdiretórios somente para agrupamentos que possuem coesão própria, como `exec`, `filter` e `fmt` [5]. Essas referências foram usadas como comparação, não como modelos copiados.

Foram deliberadamente evitados um workspace com vários crates, uma hierarquia profunda, módulos genéricos como `utils.rs` ou `helpers.rs`, uma trait para cada scheduler, factories, wrappers, duplicação de tipos e tornar tudo público para contornar erros de import. A única duplicação nominal observada na auditoria final foi `human_io_reason` em módulos de erro distintos; são funções privadas com contextos e mensagens de domínio diferentes, portanto não foi criada uma abstração artificial para eliminá-las.

## Validação realizada

A migração foi feita com compilação após os primeiros ajustes de cada fronteira. Durante a primeira tentativa, o compilador identificou visibilidades e imports insuficientes; o código gerado foi restaurado ao checkpoint, o script de migração foi corrigido para não alterar literais de struct, e a migração foi repetida de forma determinística. O resultado final passou por todas as verificações abaixo.

| Comando/fluxo | Resultado |
|---|---|
| `cargo fmt --check` | Passou. |
| `cargo check` | Passou. |
| `cargo test` | 43 passaram, 2 ignorados explicitamente, 0 falhas. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passou sem warnings. |
| `cargo check --target x86_64-pc-windows-gnu` | Passou. |
| `cargo build --release` | Passou com perfil otimizado existente. |
| `cargo audit` | Passou sem advisories bloqueantes. |
| `cargo deny check` | Passou: advisories, bans, licenses e sources aprovados. |
| `scripts/smoke_gui.sh` | O processo gráfico permaneceu ativo até o timeout esperado sob Xvfb. |
| `scripts/capture_tabs.sh` | Abriu segunda aba, alternou para a primeira, fechou a segunda e permaneceu ativo. |
| Inspeção visual | `artifacts/rovex-tabs-two.png` e `artifacts/rovex-tabs-one.png` não mostraram regressões aparentes, cortes ou controles quebrados no fluxo testado. |

O binário release foi executado no fluxo de abas, não apenas compilado. Os testes unitários continuam cobrindo seleção Ctrl/Shift/Ctrl+A, histórico e abas, filtragem, ícones semânticos, locais, carregamento real, nomes Unicode/caminhos inválidos, operações atômicas, cancelamento, segurança e descoberta de conversores.

## Conclusão

A nova arquitetura atende o objetivo principal: responsabilidades reais estão separadas em módulos coesos, a fachada desktop voltou a representar a composição da aplicação e os domínios existentes não foram fragmentados sem necessidade. O comportamento observado da v0.1.8 foi preservado, o branch principal está limpo após os commits da refatoração e existe um branch remoto de rollback no estado anterior.

A próxima melhoria funcional do issue do GitHub — acesso à Lixeira — deve ser tratada como um novo domínio, com API e testes próprios, e não incorporada artificialmente em `operations.rs` ou `desktop.rs` sem uma decisão explícita sobre a semântica multiplataforma.

## Referências

[1]: https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html "The Rust Programming Language — Control Scope and Privacy with Modules"
[2]: https://doc.rust-lang.org/cargo/guide/project-layout.html "The Cargo Book — Package Layout"
[3]: https://rust-lang.github.io/api-guidelines/about.html "Rust API Guidelines"
[4]: https://github.com/BurntSushi/ripgrep "BurntSushi/ripgrep"
[5]: https://github.com/sharkdp/fd/tree/master/src "sharkdp/fd — src"
