# Política de segurança do Rovex

## Escopo

O Rovex manipula caminhos, diretórios, arquivos locais, unidades removíveis e, futuramente, fontes de rede. Esses dados são tratados como não confiáveis. O programa não executa automaticamente arquivos encontrados durante a navegação e não envia conteúdo de arquivos para serviços externos.

## Modelo de ameaça

As principais ameaças consideradas são traversal de caminho, exclusão acidental de raízes, loops ou destinos inesperados por links e junctions, arquivos malformados, parsers vulneráveis, saturação de CPU/RAM/disco, DLL hijacking, command injection e atualização de artefatos sem integridade verificada. A arquitetura separa a UI do filesystem e usa workers nomeados para carregamentos que não devem bloquear o thread visual.

## Medidas atuais

A fundação usa APIs de arquivo diretamente, sem montar comandos shell. Destinos são validados antes de operações, a sobrescrita é recusada por padrão, a raiz não é aceita para exclusão e diretórios não vazios não são removidos pela API limitada da primeira fatia. A cópia usa arquivo temporário, sincronização e validação antes da publicação. Links simbólicos são identificados por `symlink_metadata` e não são seguidos durante a listagem.

A UI envia o trabalho de listagem a um worker e atualiza o modelo Slint somente pelo event loop. Cada carregamento recebe uma geração; quando uma navegação mais recente ocorre, o resultado anterior é descartado antes de alterar caminho, status ou linhas. O filtro local não varre subpastas: usa snapshots compartilhados e uma fila latest-only com um worker dedicado, descartando resultados obsoletos por geração. Falhas ao iniciar workers e falhas internas de atualização viram mensagens controladas, não panic.

## Dependências

A interface usa Slint 1.17.1 com backend Winit, renderer software e acessibilidade. A política `deny.toml` verifica fontes, licenças e advisories. `cargo audit` não encontrou vulnerabilidades, mas reporta advisories de manutenção transitivos para `bincode`, `paste`, `rustybuzz` e `ttf-parser`, todos introduzidos pelo toolkit atual e sem atualização segura indicada pelo RustSec no momento da verificação. O cargo-deny os trata como transitivos não bloqueantes, mantendo os avisos visíveis.

## Requisitos para novas contribuições

Novas operações precisam ter erro estruturado, teste de regressão e comportamento de cancelamento quando forem longas. Não devem introduzir `unwrap`, `expect`, `panic` ou `unsafe` em caminhos alimentados por usuário ou filesystem sem justificativa técnica. Dependências novas precisam ser mantidas, licenciadas, auditáveis e ter origem e versão documentadas.

Nenhum codec ou executável externo deve ser baixado de uma URL arbitrária em tempo de execução. Quando um backend externo for necessário, a contribuição deverá documentar origem, versão, hash, arquitetura, licença, assinatura e processo de atualização.

## Reporte

Vulnerabilidades devem ser reportadas de forma privada aos mantenedores antes de uma divulgação pública. Não inclua arquivos pessoais, tokens ou conteúdo sensível no relatório. A política será complementada com um canal específico quando a distribuição pública do aplicativo estiver estabelecida.

## Limitações conhecidas

A primeira interface ainda não contém pesquisa, abas, thumbnails, seleção avançada, drag and drop, operações de arquivo disparadas pela UI, integração com shell, conversores, OCR, instalador, assinatura ou atualização automática. A exclusão recursiva não está disponível de propósito. O build Windows foi validado de forma cruzada e a CI remota deve continuar sendo executada antes de qualquer declaração de compatibilidade final em Windows 10/11.
