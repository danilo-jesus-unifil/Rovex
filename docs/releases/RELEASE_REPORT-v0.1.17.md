# Rovex v0.1.17 — Release report

## Resumo

A versão `v0.1.17` fortalece o pipeline de distribuição portable do Rovex. O trabalho começou com uma auditoria de possíveis falhas, não com uma alteração presumida: o verificador foi submetido a um ZIP real v0.1.16 cujo manifesto havia sido adulterado. O caso passou pelo verificador antigo, confirmando uma lacuna de integridade de metadados. A correção exige coerência entre o nome do artefato, a raiz do ZIP, a versão do manifesto e o target Windows GNU, e foi incorporada ao CI com testes adversariais.

| Item | Resultado |
|---|---|
| Versão | `0.1.17` |
| Commit de hardening | `1d5077a fix(release): validate portable manifest metadata` |
| Verificador principal | `scripts/verify_windows_portable.sh` |
| Teste adversarial | `scripts/test_verify_windows_portable.sh` |
| Job CI | `Windows GNU cross-build` empacota e testa o portable |
| Artefato | ZIP Windows x86-64 portable, sem assinatura e sem downloads em runtime |

## Auditoria e falha confirmada

O estado inicial estava limpo, sincronizado com `origin/main`, na tag `v0.1.16`, com CI verde e release publicada. A auditoria procurou divergências em scripts, versões, documentação e empacotamento. Os scripts shell passaram por `bash -n`, o audit CLI existente passou e a presença de Open With no main foi confirmada; portanto, esses riscos foram descartados como não reproduzidos.

A hipótese verificável foi a possibilidade de o verificador aceitar um pacote cujo nome externo dissesse uma versão, mas cujo `DISTRIBUTION-MANIFEST.txt` registrasse outra. Uma cópia real do ZIP v0.1.16 foi extraída, recebeu `version=9.9.9`, foi reempacotada e recebeu SHA-256 novo. O verificador anterior aceitou esse pacote adulterado. O resultado foi uma falha confirmada, não uma preocupação teórica.

> A existência de um SHA-256 válido não prova a coerência semântica do conteúdo: ele só prova a integridade da cópia em relação ao checksum fornecido.

## Correção implementada

`verify_windows_portable.sh` agora extrai o nome-base do ZIP, exige que a primeira raiz do arquivo seja exatamente essa base e deriva a versão esperada de `rovex-v<version>-windows-x86_64-portable.zip`. O arquivo `DISTRIBUTION-MANIFEST.txt` precisa conter exatamente `version=<version>` e `target=x86_64-pc-windows-gnu`. Os arquivos obrigatórios continuam sendo verificados antes da leitura do manifesto, preservando diagnósticos corretos para pacotes incompletos.

O novo `test_verify_windows_portable.sh` primeiro verifica o pacote original e depois cria três fixtures temporários: versão adulterada, target adulterado e raiz divergente. Cada fixture precisa ser rejeitado; se algum for aceito, o teste termina com falha. O teste não modifica o artefato original e remove o workspace temporário ao finalizar.

O job `Windows GNU cross-build` agora executa `package_windows_portable.sh` e o teste adversarial em checkout limpo. Assim, a validação de distribuição não fica restrita à máquina que publica a release e passa a ser uma regressão automática do pipeline.

## Evidências de validação

| Verificação | Resultado |
|---|---|
| ZIP v0.1.16 original com verificador novo | Aprovado |
| Fixture com `version=9.9.9` | Rejeitado |
| Fixture com target divergente | Rejeitado |
| Fixture com raiz divergente | Rejeitado |
| `bash -n` em scripts shell | Aprovado |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --all-targets --all-features` | Aprovado |
| `cargo test --all-targets --all-features` | 97 aprovados; 2 ignorados |
| Clippy host e Windows GNU | Aprovado sem warnings |
| Link dos testes Windows GNU | Aprovado |
| CI do commit `1d5077a` | Ubuntu, Windows nativo, cross-Windows, auditoria e Documentation layout passaram |

## Limitações honestas

A validação garante coerência dos metadados básicos do ZIP portable, mas não assina o executável, não prova autenticidade do distribuidor e não substitui a verificação externa do SHA-256 por um canal confiável. O pacote continua sem instalador, sem assinatura Authenticode e sem atualização automática. A execução interativa em Windows 10/11, incluindo Explorer, ACLs, DPI, acessibilidade, Shell, UNC/SMB, reparse points e arquivos em uso, permanece um gate separado.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/dlgbox/using-the-common-dialog-box-library "Microsoft Common Dialog Box Library guidance"

[2]: https://docs.github.com/en/actions "GitHub Actions documentation"
