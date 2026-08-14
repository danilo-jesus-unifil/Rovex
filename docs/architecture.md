# Rovex — Arquitetura técnica inicial

## 1. Objetivo e interpretação do requisito

O Rovex será um explorador de arquivos para Windows 10 e 11, escrito prioritariamente em Rust, com processamento local, operações reais sobre o sistema de arquivos e conversores isolados. O termo **nativo** será usado com precisão: o programa será um binário desktop compilado para Windows, integrado às APIs do sistema e sem Electron ou uma interface web empacotada. A camada visual, entretanto, será renderizada pelo toolkit escolhido, e não será apresentada como um conjunto de controles Win32 nativos sem que isso seja efetivamente implementado.

O prompt master mistura um produto de exploração de arquivos com um conjunto amplo de conversores multimídia, OCR, PDF, integração de shell, instalador, atualização e validação de segurança. Implementar tudo de uma vez aumentaria de forma desnecessária a superfície de ataque e impediria validar a fundação. Por isso, a primeira entrega deve privilegiar a fundação segura e funcional do explorador; conversores e recursos de maior risco entrarão somente depois de contratos, limites e workers isolados estarem testados.

## 2. Decisão provisória de interface

A decisão atual é **Slint**, por ser um toolkit orientado a Rust para interfaces desktop compiladas, com integração a APIs do sistema e uma proposta de runtime compacto [1]. A primeira janela foi implementada com backend Winit, renderer software, acessibilidade, barra de endereço, lista real e carregamento em workers. A escolha ainda precisa ser validada em Windows 10/11 com teclado, leitor de tela, escalas de 100% a 200%, múltiplos monitores e modo escuro/claro. A documentação do Slint recomenda declarar explicitamente papel, rótulo e ações de acessibilidade em componentes personalizados [2].

| Alternativa | Vantagem principal | Risco para este projeto | Decisão |
|---|---|---|---|
| Slint | Integração Rust, UI declarativa compilada e baixo acoplamento entre UI e domínio | Controles não são automaticamente equivalentes a controles Win32; acessibilidade deve ser validada | **Escolhida para a primeira UI** |
| Iced | Ecossistema Rust e arquitetura declarativa | Renderização e acessibilidade precisam ser avaliadas contra o requisito do Explorer | Reserva |
| egui | Prototipagem rápida e baixo custo inicial | Menor adequação para uma experiência de explorador acessível e refinada | Não escolhida |
| Win32 direto via `windows-rs` | Integração máxima com Windows e controles nativos | Complexidade, maior custo de manutenção e UI moderna mais trabalhosa | Reserva para componentes específicos |
| Tauri/Electron | Grande disponibilidade de componentes web | Contraria a proibição explícita de webview empacotada/Electron | Rejeitada |

A Microsoft recomenda declarar o DPI padrão no manifesto do processo, preferencialmente usando `<dpiAwareness>` com fallback para versões compatíveis, em vez de configurar essa propriedade por API depois da criação de janelas [3]. O manifesto do Rovex seguirá esse modelo e a validação será feita em ambiente Windows, não apenas em Linux.

## 3. Limites da primeira entrega

A primeira fatia funcional entregue nesta etapa tem janela única, navegação por caminho e pasta pai, listagem real de arquivos, ativação de diretórios, atualização, status e erros controlados. O núcleo também possui criação de pasta, renomeação, cópia atômica e exclusão limitada, ainda sem comandos visuais na UI. A interface não executará arquivos automaticamente nem carregará parsers multimídia dentro do processo principal.

Pesquisa indexada, thumbnails, abas, conversão em lote e integrações de shell serão adicionadas em etapas separadas. O suporte a PDF, OCR, áudio e vídeo não será simulado: cada recurso somente aparecerá como concluído quando houver backend verificável, worker isolado, testes adversariais e validação do arquivo gerado.

## 4. Camadas e contratos

A UI consumirá comandos e eventos de domínio; não conhecerá caminhos internos de codecs, códigos brutos do Win32 ou detalhes de implementação do sistema de arquivos. O fluxo principal será:

```text
UI Slint
  ↓ comandos tipados
Application/Core
  ↓ portas e eventos
Filesystem Adapter ── Windows Compatibility Layer
  ↓
Sistema de arquivos local, USB, UNC e rede
```

Os módulos iniciais serão organizados como segue:

| Módulo | Responsabilidade | Não deve fazer |
|---|---|---|
| `app` | Ciclo de vida, composição e roteamento de comandos | Acessar APIs Win32 diretamente na UI |
| `core` | Tipos de domínio, seleção, navegação e estados | Conhecer detalhes de widgets |
| `filesystem` | Leitura e operações reais de arquivos e diretórios | Executar conteúdo do usuário |
| `operations` | Filas, progresso, cancelamento e conflitos | Declarar sucesso antes da verificação |
| `windows_compat` | Manifesto, erros Win32 e comportamentos específicos | Espalhar condicionais de Windows pelo domínio |
| `ui` | Apresentação, comandos e acessibilidade | Fazer cópia, parsing ou conversão síncrona |
| `logging` | Logs redigidos e diagnóstico local | Registrar conteúdo, tokens ou dados pessoais |
| `converters` | Contratos de conversão e workers futuros | Rodar codecs no processo da UI |
| `security` | Validações de destino, links e limites | Substituir a autorização do sistema operacional |

## 5. Modelo de segurança

Todos os nomes, caminhos, links simbólicos, junctions, pontos de reparse, arquivos baixados, mídias, PDFs, metadados e arquivos de rede serão tratados como entradas não confiáveis. Operações destrutivas validarão tipo, existência, raiz, destino esperado e comportamento de links. A resolução de destino deverá impedir traversal e não seguirá links recursivamente sem uma política explícita.

O programa será executado como usuário normal, com manifesto `asInvoker`. Não haverá elevação global para administrador. Conversores e parsers de maior risco utilizarão workers com limites de CPU, memória, tamanho de entrada, tempo e número de tarefas. O processo principal somente receberá resultados estruturados e mensagens de erro redigidas.

A estratégia de escrita para arquivos gerados será temporária e verificável: criar o arquivo temporário no destino apropriado, gravar e fechar, validar o resultado e só então renomear atomicamente para a extensão final quando a operação permitir. O original permanecerá preservado por padrão.

## 6. Política de erros e observabilidade

Erros serão representados por tipos estruturados, incluindo `FileOperationError`, `ValidationError`, `PermissionError`, `WindowsError` e, posteriormente, `ConversionError`. A interface mostrará mensagens acionáveis e o diagnóstico preservará o código técnico relevante. `unwrap`, `expect` e `panic` não serão usados em caminhos alimentados por filesystem, usuário, arquivos externos ou codecs sem uma justificativa e um teste correspondente.

Os logs serão locais, com níveis configuráveis e redação de informações sensíveis. Por padrão, o Rovex não enviará telemetria nem conteúdo de arquivos. Identificadores de operação e métricas de desenvolvimento poderão existir internamente, mas a versão de distribuição deverá reduzir logs e evitar caminhos completos quando não forem necessários.

## 7. Estratégia de testes

A matriz de testes começará por testes unitários de validação de caminhos, seleção, conflitos e transições de estado. Em seguida serão adicionados testes de filesystem em diretórios temporários, testes de interrupção, permissões simuladas quando possível, nomes Unicode, paths longos, links e erros de disco. O ambiente Linux não substitui a validação final: o CI deverá incluir Windows x64, debug e release.

O ciclo de aceite da primeira fatia é:

```text
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo audit
cargo deny check
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
Smoke UI em display virtual
Windows: execução manual, DPI, teclado, operações e instalação
```

Resultados que não possam ser verificados no ambiente atual serão registrados como **não verificados**, jamais como aprovados.

## 8. Riscos e decisões pendentes

O maior risco técnico é a combinação entre UI moderna, acessibilidade consistente e compatibilidade com Windows 10. O protótipo precisa resolver esse risco antes de receber recursos de conversão. O segundo risco é a expectativa de “baixo consumo”: o número declarado para o runtime do framework não representa o consumo total de uma aplicação com thumbnails, cache, workers e listagens extensas. Por isso, o projeto terá benchmarks de inicialização, idle, 10 mil itens, cache e conversão antes de estabelecer metas.

Também será necessário decidir, em uma etapa posterior, como distribuir codecs. Nenhum executável externo será baixado de URL arbitrária em tempo de execução. Qualquer backend como FFmpeg precisará de versão, origem, hash, arquitetura, licença, processo de atualização e verificação de integridade documentados.

## Referências

[1]: https://slint.dev/ "Slint — página oficial"
[2]: https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/ "Slint Docs — Best Practices"
[3]: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process "Microsoft Learn — Setting the default DPI awareness for a process"
