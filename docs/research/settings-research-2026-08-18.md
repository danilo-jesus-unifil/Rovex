# Pesquisa de configurações persistentes — 2026-08-18

## Decisão de armazenamento

O Rovex é um aplicativo desktop unpackaged, compilado com Slint/Winit, e não possui acesso ao armazenamento gerenciado de aplicativos empacotados. A documentação da Microsoft diferencia explicitamente esses cenários e orienta aplicativos unpackaged a usar I/O de arquivo direto ou o Registro; para preferências e estado específico do aplicativo, o armazenamento local por usuário é o local conceitualmente adequado [1].

| Dado | Local do Rovex | Motivo |
|---|---|---|
| Configurações do aplicativo | `%LOCALAPPDATA%\\Rovex\\settings.v1.conf` no Windows | Não exige privilégios e não grava no diretório de instalação. |
| Fallback Unix de desenvolvimento | `$XDG_CONFIG_HOME/Rovex` ou `$HOME/.config/Rovex` | Mantém os smoke tests isolados sem introduzir lógica Windows no host Linux. |
| Override de teste | `ROVEX_CONFIG_DIR/Rovex/settings.v1.conf`, apenas quando absoluto | Permite testes determinísticos sem modificar a configuração real do usuário. |

O arquivo guarda somente preferências do Rovex: último diretório válido, visibilidade de ocultos e coluna/direção de ordenação. Não armazena conteúdo dos arquivos, credenciais, caminhos de conversão ou dados que o usuário possa considerar irrecuperáveis. O path é codificado em hexadecimal sobre a representação nativa do sistema para preservar nomes Unicode e, no Windows, unidades UTF-16 que não dependem de conversão lossy.

## Formato e tolerância a falhas

O schema é versionado (`version=1`) e limitado a 16 KiB. O parser exige chaves essenciais, rejeita valores inválidos, caminhos relativos, duplicatas, alvos que sejam symlinks ou diretórios, e ignora chaves desconhecidas para permitir evolução compatível. Arquivo ausente, corrompido ou inacessível não impede a inicialização: o Rovex usa defaults seguros e registra um aviso local.

A escrita usa arquivo temporário criado com `create_new`, `write_all`, `sync_all` e substituição no mesmo diretório. Em Unix, `rename` mantém a operação no mesmo filesystem. No Windows, `MoveFileExW` recebe `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`; a documentação oficial define o primeiro como substituição do destino existente e o segundo como espera até a operação ser descarregada para o disco [2]. O temporário é removido quando a operação falha.

## Evidências

Os testes unitários cobrem round-trip com Unicode, segunda gravação sem temporário órfão, arquivo ausente/corrompido, limite de tamanho e chave futura. `scripts/test_settings.sh` alterna Ocultos, seleciona a coluna Modificado, verifica o arquivo no diretório de teste e relança o binário com o mesmo store; a segunda UI permanece ativa e lê o estado persistido. A matriz de validação também deve incluir `cargo check --target x86_64-pc-windows-gnu`, pois a substituição Windows é compilada apenas nesse alvo.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/apps/develop/data/store-and-retrieve-app-data "Store and retrieve settings and other app data — Microsoft Learn"

[2]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexa "MoveFileExA function — Microsoft Learn"
