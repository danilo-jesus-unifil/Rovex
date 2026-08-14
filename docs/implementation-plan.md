# Rovex — Plano incremental de implementação

## Princípio de execução

Cada etapa precisa produzir uma fatia real, compilável e testável. O projeto não deverá anunciar funcionalidades futuras como se já estivessem disponíveis. A ordem abaixo prioriza segurança, correção, estabilidade e compatibilidade com Windows 10 antes de aparência e amplitude funcional.

| Etapa | Entrega verificável | Critério de aceite |
|---|---|---|
| 0. Fundação | Cargo workspace, toolchain, linting, CI e documentação | Build limpo, format check, testes mínimos e licença registrada |
| 1. Domínio | Tipos de caminho, item de arquivo, seleção, erros e navegação | Testes unitários cobrindo estados válidos e inválidos |
| 2. Filesystem | Listagem, drives, diretórios, operações básicas e conflitos | Testes com temporários, Unicode, arquivos grandes e erros controlados |
| 3. UI | Janela, barra de endereço, lista, seleção e comandos reais | Navegação funcional sem bloquear a UI; teclado básico |
| 4. Segurança operacional | Validação de destino, links, cancelamento, atomicidade e logs | Testes adversariais e ausência de sucesso falso |
| 5. Pesquisa e cache | Pesquisa cancelável, resultados incrementais e cache limitado | Stress test com grandes diretórios e limites de recursos |
| 6. Thumbnails | Workers limitados, cache e fallback genérico | Arquivo malformado não derruba o processo principal |
| 7. Abas e UX | Abas, histórico, atalhos, tema e acessibilidade | Teste com teclado, DPI e múltiplos monitores em Windows |
| 8. Conversores | Imagem, documentos e depois áudio/vídeo em workers isolados | Backend versionado, validação de saída, cancelamento e testes adversariais |
| 9. Distribuição | MSIX ou instalador alternativo, portable, assinatura e rollback | Instalação, atualização, desinstalação e verificação de artefatos |
| 10. Release | Auditoria final, benchmarks e relatórios | `RELEASE_REPORT.md`, `SECURITY.md` e lista de limitações atualizados |

## Escopo imediato

A próxima alteração de código deverá implementar a fundação e o domínio, não os conversores. O objetivo é criar uma base pequena que possa ser compilada e testada mesmo antes da UI completa. A camada de filesystem será introduzida por uma porta testável, permitindo que o domínio seja validado com implementações temporárias e evitando que os testes dependam de uma máquina Windows específica.

## Definição de pronto da primeira versão de desenvolvimento

A primeira versão de desenvolvimento será considerada pronta somente quando listar um diretório real, permitir navegar para um subdiretório, retornar erros estruturados e executar ao menos uma operação de arquivo real sem bloquear a interface. Antes disso, qualquer tela visual será considerada protótipo técnico e deverá ser identificada como tal.

## Itens explicitamente fora da primeira fatia

A primeira fatia não incluirá execução automática de arquivos, integração silenciosa com o shell, associação de arquivos, atualização automática, OCR, conversão de PDF, FFmpeg, codecs externos, thumbnails complexas ou exclusão permanente sem confirmação. Esses recursos dependem de decisões de segurança e distribuição que não devem ser mascaradas por uma interface inicial.
