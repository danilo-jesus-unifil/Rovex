# Rovex — Plano incremental de implementação

## Princípio de execução

Cada etapa precisa produzir uma fatia real, compilável e testável. O projeto não deverá anunciar funcionalidades futuras como se já estivessem disponíveis. A ordem abaixo prioriza segurança, correção, estabilidade e compatibilidade com Windows 10 antes de aparência e amplitude funcional.

| Etapa | Entrega verificável | Critério de aceite |
|---|---|---|
| 0. Fundação | Cargo workspace, toolchain, linting, CI e documentação | Build limpo, format check, testes mínimos e licença registrada |
| 1. Domínio | Tipos de caminho, item de arquivo, seleção, erros e navegação | Testes unitários cobrindo estados válidos e inválidos |
| 2. Filesystem | Listagem, drives, diretórios, operações básicas e conflitos | Testes com temporários, Unicode, arquivos grandes e erros controlados |
| 3. UI | Janela Slint, barra de endereço, lista, atualização, pasta pai e filtro local | Navegação e filtro funcionais sem bloquear a UI; smoke test visual |
| 4. Segurança operacional | Validação de destino, links, cancelamento, atomicidade e logs | Testes adversariais e ausência de sucesso falso |
| 5. Pesquisa e cache | Filtro local latest-only concluído; pesquisa global cancelável e cache limitado ainda pendentes | Stress test com grandes diretórios e limites de recursos |
| 6. Thumbnails | Workers limitados, cache e fallback genérico | Arquivo malformado não derruba o processo principal |
| 7. Abas e UX | Abas, histórico, atalhos, tema e acessibilidade | Teste com teclado, DPI e múltiplos monitores em Windows |
| 8. Conversores | Imagem, documentos e depois áudio/vídeo em workers isolados | Backend versionado, validação de saída, cancelamento e testes adversariais |
| 9. Distribuição | MSIX ou instalador alternativo, portable, assinatura e rollback | Instalação, atualização, desinstalação e verificação de artefatos |
| 10. Release | Auditoria final, benchmarks e relatórios | `RELEASE_REPORT.md`, `SECURITY.md` e lista de limitações atualizados |

## Escopo imediato

A próxima alteração de código deve continuar a partir da UI funcional sem antecipar conversores. A prioridade imediata é fortalecer histórico, seleção e operações visuais somente quando existirem contratos de cancelamento e confirmação. Pesquisa global, thumbnails, análise de espaço e conversores permanecem sob demanda, em workers limitados, com testes de estresse e falha.

## Definição de pronto da primeira versão de desenvolvimento

A primeira versão de desenvolvimento atual lista um diretório real, permite navegar para um subdiretório, retorna erros estruturados, filtra a pasta atual sem pesquisa recursiva e mantém o event loop livre durante o filtro. Operações de arquivo reais já existem no núcleo, mas ainda não estão expostas por comandos visuais.

## Itens explicitamente fora da primeira fatia

A primeira fatia não incluirá execução automática de arquivos, integração silenciosa com o shell, associação de arquivos, atualização automática, OCR, conversão de PDF, FFmpeg, codecs externos, thumbnails complexas ou exclusão permanente sem confirmação. Esses recursos dependem de decisões de segurança e distribuição que não devem ser mascaradas por uma interface inicial.
