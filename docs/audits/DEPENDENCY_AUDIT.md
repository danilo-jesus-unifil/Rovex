# Auditoria e racionalização de dependências do Rovex

**Data:** 15 de agosto de 2026
**Projeto:** Rovex: explorador de arquivos local em Rust/Slint
**Escopo:** reduzir o custo de dependências sem remover bibliotecas essenciais, sem reimplementar componentes complexos e sem alterar a compatibilidade Windows 10/11.

> **Conclusão executiva:** o Rovex já possuía apenas uma dependência direta de runtime e uma dependência direta de build. Não havia crate redundante do projeto para remover com segurança. A única consolidação comprovada foi tornar a feature de backend Winit específica por plataforma: Linux usa somente X11, necessário para o CI com Xvfb; Windows 10/11 mantém o backend Winit completo. A árvore efetivamente compilada no Linux diminuiu, o binário Linux ficou menor e o caminho Windows permaneceu funcional e do mesmo tamanho.

## 1. Método

A auditoria analisou `Cargo.toml`, `Cargo.lock`, `cargo metadata`, `cargo tree` com dependências normais e de build, duplicações por nome e versão, features ativadas, manifests locais do Slint 1.17.1, build release, build cruzado Windows, testes, Clippy, `cargo audit`, `cargo deny` e smoke visual real em Xvfb. A comparação usa o commit publicado `8d5805f` como baseline e a alteração não publicada deste incremento como estado posterior.

A análise não substituiu uma biblioteca madura por código próprio. Também não alterou a versão do Slint, não removeu acessibilidade, não removeu o renderer software, não adicionou uma segunda stack de UI e não introduziu dependência direta nova.

## 2. Matriz de dependências diretas

| Dependência | Categoria | Uso real | Features | Classificação | Decisão |
|---|---|---|---|---|---|
| `slint = 1.17.1` | Runtime | Janela desktop, event loop, modelos `VecModel`, acessibilidade, layout e renderer software | `backend-winit-x11` no Linux; `backend-winit` em Windows e Unix não-Linux; `renderer-software`, `accessibility`, `compat-1-2` | **Essencial** | Manter; restringir somente o backend Linux ao alvo efetivo |
| `slint-build = 1.17.1` | Build | Compilar `ui/main.slint` no `build.rs` | Defaults do crate, sem feature opcional | **Essencial de build** | Manter |

O projeto não possui dependências em `[dev-dependencies]`. As ferramentas de auditoria, compilação e testes são executadas no ambiente de desenvolvimento/CI e não entram no binário distribuído. Não há dependência direta de filesystem, regex, serialização, rede, banco de dados, logging, compressão, multimídia ou ícones.

## 3. Alteração aplicada

Antes, o manifesto aplicava a mesma feature `backend-winit` a todos os alvos. No Slint 1.17.1, essa feature do selector ativa tanto `backend-winit-x11` quanto `backend-winit-wayland`. O Rovex tem como alvo principal Windows 10/11 e o CI Linux usa Xvfb/X11. Portanto, a aplicação segura foi separar o manifesto por plataforma:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
slint = { version = "=1.17.1", default-features = false, features = [
    "backend-winit-x11", "renderer-software", "accessibility", "compat-1-2"
] }

[target.'cfg(windows)'.dependencies]
slint = { version = "=1.17.1", default-features = false, features = [
    "backend-winit", "renderer-software", "accessibility", "compat-1-2"
] }
```

Unix não-Linux preserva o backend completo para não restringir plataformas que não foram validadas nesta etapa. A feature `compat-1-2` foi mantida porque o próprio Slint exige sua ativação quando os defaults são desabilitados. `accessibility` foi mantida porque a UI declara papéis e rótulos acessíveis. `renderer-software` foi mantida porque é a escolha deliberada de baixo risco e compatibilidade do projeto.

A alteração **não removeu crates do lockfile global**. Isso é esperado: o lockfile continua descrevendo dependências de Windows, Linux e outras plataformas. A redução ocorre no grafo efetivamente compilado para o alvo Linux.

## 4. Árvore e duplicações

| Métrica | Baseline | Depois | Variação | Interpretação |
|---|---:|---:|---:|---|
| Pacotes resolvidos em `cargo metadata` | 587 | 587 | 0 | O lockfile global ainda precisa representar todos os alvos |
| Nós `cargo tree -e normal` no host Linux | 1.047 | 913 | **-134 (-12,8%)** | Redução do backend Wayland no grafo Linux efetivo |
| Nós `cargo tree -e all` no host Linux | 3.083 | 2.738 | **-345 (-11,2%)** | Menos nós normais/features no build Linux |
| Nós normais Linux por alvo | 1.275 | 1.140 | **-135 (-10,6%)** | Comparação específica `x86_64-unknown-linux-gnu` |
| Pacotes únicos Linux por alvo | 969 | 850 | **-119 (-12,3%)** | Pacotes efetivamente necessários ao alvo Linux |
| Nós normais Windows por alvo | 892 | 892 | 0 | Compatibilidade Windows preservada |
| Pacotes únicos Windows por alvo | 641 | 641 | 0 | Backend Windows não foi reduzido |
| Nomes com múltiplas versões no lockfile | 44 | 44 | 0 | Duplicações são transitivas do toolkit/plataformas |
| Tamanho de `Cargo.lock` | 150.741 bytes | 150.741 bytes | 0 | Não houve remoção global artificial |

As duplicações observadas não são redundâncias introduzidas pelo Rovex. Exemplos incluem `tiny-skia` 0.11 e 0.12 trazidos por `sctk-adwaita`/Winit e por `resvg`/fontes do Slint; `thiserror` 1 e 2 em ramos diferentes do toolkit; `syn` 2 e 3 em proc-macros; e várias versões de `windows-sys` para APIs distintas. Forçar essas versões por patches ou dependências diretas aumentaria o risco e não atenderia à regra de equivalência do prompt.

## 5. Features auditadas

| Feature | Estado | Motivo |
|---|---|---|
| `default-features = false` | Mantida | Evita `renderer-femtovg` e outros defaults não usados pelo produto |
| `backend-winit-x11` | Nova no Linux | O CI e os smoke tests Linux usam X11/Xvfb; Wayland não é necessário nesse alvo |
| `backend-winit` | Mantida em Windows/Unix não-Linux | Preserva o backend Winit completo fora do Linux validado |
| `renderer-software` | Mantida | Compatibilidade e previsibilidade, sem dependência de OpenGL/Vulkan |
| `accessibility` | Mantida | Necessária para os papéis e rótulos acessíveis da UI |
| `compat-1-2` | Mantida | Feature obrigatória do Slint quando defaults são desabilitados |
| `system-tray`, `renderer-femtovg`, `renderer-skia`, `live-preview`, `mcp`, `serde`, `gettext`, `image-default-formats` | Não ativadas | Não são utilizadas pelo Rovex atual |

Não foram criadas features artificiais para recursos ainda ausentes. Pesquisa global, thumbnails, preview, conversores, compactação, hash e rede continuam fora do binário até que exista uma implementação real sob demanda.

## 6. Métricas de build, binário e runtime

| Métrica | Baseline | Depois | Observação |
|---|---:|---:|---|
| Clean build Linux release | 149,09 s | 131,63 s | **-17,46 s (-11,7%)** na mesma máquina; resultado observado, não promessa universal |
| Pico RSS do processo de build Linux | 946.316 KiB | 949.952 KiB | Variação de ambiente/ordenação; não houve ganho de memória comprovado |
| Binário Linux release | 18.002.864 bytes | 16.128.776 bytes | **-1.874.088 bytes (-10,4%)** |
| Binário Windows release | 12.488.704 bytes | 12.488.704 bytes | Sem alteração; backend Windows preservado |
| CLI em 10.000 arquivos | 0,0269 s / 122.412 KiB RSS | 0,0432 s / 123.476 KiB RSS | Amostras isoladas com variação; nenhuma alegação de ganho de runtime |
| Build incremental Linux | 1,72 s | Não refeito após o clean final | O baseline incremental foi registrado antes da alteração; a comparação limpa é a métrica principal |

O clean build Windows baseline em cópia isolada terminou em aproximadamente 4m48s, enquanto um build posterior do estado reduzido terminou em aproximadamente 3m02s com cache e ordem de compilação diferentes. Essa comparação não é tratada como ganho válido de build; a métrica confiável desta etapa é a redução observada no alvo Linux, onde a alteração efetivamente removeu o backend Wayland.

## 7. Segurança, compatibilidade e riscos

A alteração mantém o mesmo Slint fixado, o mesmo renderer software, os mesmos recursos de acessibilidade e o mesmo código Rust. O build Windows cruzado `x86_64-pc-windows-gnu` passou após a separação, e o artefato continuou PE32+ x86-64 com o mesmo tamanho observado.

O risco introduzido é explícito: a build Linux desta configuração passa a exigir X11 e não inclui backend Wayland. Isso é aceitável para o escopo atual porque o alvo de produto é Windows 10/11 e o CI/smoke Linux é baseado em Xvfb/X11. Se suporte Linux Wayland se tornar requisito, deve-se criar uma feature/target Linux separado ou restaurar `backend-winit`; não se deve esconder essa limitação.

Não foram introduzidos `unsafe`, dependências diretas novas, parsers próprios, código de criptografia, cópia de crates de terceiros ou mudanças nas invariantes de filesystem. O backup remoto criado antes da limpeza é `backup/before-dependency-cleanup-2026-08-15`.

## 8. Validação executada

A cópia experimental com a mesma alteração passou `cargo fmt --check`, `cargo check --all-targets --all-features`, `cargo test --all-targets --all-features`, Clippy, build release Linux, build cruzado Windows e smoke visual Xvfb. O projeto principal também passou check e testes após a alteração, build release Linux e build cruzado Windows com `CARGO_BUILD_JOBS=2`.

A validação final de supply chain deve incluir, antes do commit, `cargo audit` e `cargo deny check`. O ambiente não possui `cargo-outdated` instalado; portanto, nenhuma saída de atualização foi simulada. A ausência da ferramenta será registrada se permanecer assim durante a validação final.

## 9. Decisões não aplicadas

Não removi `slint`, porque ele é a única stack de UI, event loop, modelo e acessibilidade do produto. Não removi `slint-build`, porque o `build.rs` compila a interface e substituí-lo por parsing próprio seria código complexo e frágil. Não forcei a unificação das versões duplicadas do lockfile, porque são transitivas, têm origens diferentes e a unificação poderia quebrar compatibilidade do Slint/Winit ou de plataformas. Não removi `accessibility`, `renderer-software` ou `compat-1-2`, pois isso sacrificaria requisitos explícitos ou quebraria a configuração suportada.

> A conclusão desta auditoria não é “a árvore global ficou menor a qualquer custo”. É que o conjunto direto já era mínimo, e a única redução comprovada sem sacrificar Windows 10/11 foi excluir do build Linux uma plataforma gráfica que o alvo Linux validado não usa.

## 10. Referências

[1]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies "Cargo: Platform-specific dependencies"
[2]: https://doc.rust-lang.org/cargo/reference/features.html "Cargo: Features"
[3]: https://doc.rust-lang.org/cargo/commands/cargo-tree.html "Cargo: cargo tree"
[4]: https://docs.rs/slint/1.17.1/slint/ "Slint 1.17.1 API documentation"
[5]: https://docs.rs/slint-build/1.17.1/slint_build/ "slint-build 1.17.1 API documentation"
[6]: https://doc.rust-lang.org/cargo/commands/cargo-metadata.html "Cargo: cargo metadata"
