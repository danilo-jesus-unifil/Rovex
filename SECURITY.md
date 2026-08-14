# Política de segurança do Rovex

## Escopo

O Rovex manipula caminhos, diretórios, arquivos locais, unidades removíveis e, futuramente, fontes de rede. Esses dados são tratados como não confiáveis. O programa não executa automaticamente arquivos encontrados durante a navegação e não envia conteúdo de arquivos para serviços externos.

## Modelo de ameaça

As principais ameaças consideradas são traversal de caminho, exclusão acidental de raízes, loops ou destinos inesperados por links e junctions, arquivos malformados, parsers vulneráveis, saturação de CPU/RAM/disco, DLL hijacking, command injection e atualização de artefatos sem integridade verificada. A arquitetura separa a UI do filesystem e reserva workers isolados para parsers e conversores de maior risco.

## Medidas atuais

A fundação atual usa APIs de arquivo diretamente, sem montar comandos shell. Destinos são validados antes de operações, a sobrescrita é recusada por padrão, a raiz não é aceita para exclusão e diretórios não vazios não são removidos pela API limitada da primeira fatia. A cópia usa arquivo temporário, sincronização e validação antes da publicação. Links simbólicos são identificados por `symlink_metadata` e não são seguidos durante a listagem.

## Requisitos para novas contribuições

Novas operações precisam ter erro estruturado, teste de regressão e comportamento de cancelamento quando forem longas. Não devem introduzir `unwrap`, `expect`, `panic` ou `unsafe` em caminhos alimentados por usuário ou filesystem sem justificativa técnica. Dependências novas precisam ser mantidas, licenciadas e auditáveis.

Nenhum codec ou executável externo deve ser baixado de uma URL arbitrária em tempo de execução. Quando um backend externo for necessário, a contribuição deverá documentar origem, versão, hash, arquitetura, licença, assinatura e processo de atualização.

## Reporte

Vulnerabilidades devem ser reportadas de forma privada aos mantenedores antes de uma divulgação pública. Não inclua arquivos pessoais, tokens ou conteúdo sensível no relatório. A política será complementada com um canal específico quando a distribuição pública do aplicativo estiver estabelecida.

## Limitações conhecidas

A primeira fatia ainda não contém UI desktop, integração com shell, pesquisa, thumbnails, conversores, OCR, instalador, assinatura ou atualização automática. A exclusão recursiva não está disponível de propósito. O CI e os testes em Windows ainda precisam ser adicionados antes de qualquer declaração de compatibilidade final.
