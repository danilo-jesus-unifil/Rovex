# Pesquisa de distribuição Windows — 2026-08-19

## Assinatura e SmartScreen

A Microsoft documenta o `SignTool` como utilitário para assinar, verificar e aplicar timestamp a arquivos; versões atuais exigem `/fd` e `/td`, recomendando SHA-256 [1]. A assinatura protege contra adulteração e permite verificar o publicador, mas requer um certificado real e uma cadeia de confiança; o Rovex não possui certificado ou segredo de assinatura nesta sessão.

O SmartScreen avalia reputação do publicador e do hash do arquivo. Um binário novo pode gerar aviso mesmo quando assinado, enquanto um binário sem assinatura começa sem reputação e pode ser bloqueado por políticas corporativas; não é correto prometer “sem alerta” apenas por gerar um EXE [2]. O pacote será, portanto, marcado explicitamente como **unsigned / não assinado** até existir um fluxo de assinatura real. Não será usado certificado autoassinado para simular confiança.

## Portable e GitHub Releases

O pacote portable será um ZIP contendo o EXE Windows GNU release, `LICENSE`, um manifesto de distribuição e uma cópia do README/compatibilidade. O pacote não instala serviço, não grava no diretório de instalação e não baixa executáveis em runtime; os backends FFmpeg continuam opcionais e externos, conforme a documentação do projeto.

O `gh release upload` aceita múltiplos assets e `--clobber` substitui arquivos existentes, o que exige cuidado para não sobrescrever release incorreta [3]. O GitHub também expõe digests SHA-256 imutáveis dos assets enviados, disponíveis na UI, API e CLI [4]. O pipeline local ainda gerará `SHA256SUMS` determinístico para permitir verificação independente antes ou fora do GitHub.

| Item | Política do Rovex |
|---|---|
| Artefato | `rovex-v<VERSION>-windows-x86_64-portable.zip` |
| Integridade | `sha256sum` sobre ZIP e manifesto de checksums incluído no release |
| Assinatura | Não assinada até haver certificado/SignTool configurado; nunca declarar o contrário |
| Instalação | Extrair para diretório escolhido pelo usuário; sem privilégios e sem instalador EXE nesta etapa |
| Runtime | Nenhum download/execução de ferramenta externa; FFmpeg é dependência opcional documentada |
| Publicação | Release GitHub somente com tag/artefatos conferidos; upload automatizado posterior não será executado sem confirmação para uma operação sensível |

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool "SignTool — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation "SmartScreen reputation for Windows app developers — Microsoft Learn"

[3]: https://cli.github.com/manual/gh_release_upload "gh release upload — GitHub CLI manual"

[4]: https://github.blog/changelog/2025-06-03-releases-now-expose-digests-for-release-assets/ "Releases now expose digests for release assets — GitHub Changelog"
