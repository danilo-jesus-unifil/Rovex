# Notas de pesquisa — decisões iniciais

## Slint
- Fonte consultada: https://slint.dev/
- A página oficial descreve o Slint como toolkit declarativo para Rust, C++, JavaScript e Python, voltado a interfaces nativas para desktop, mobile e embedded.
- A proposta destaca acesso a APIs do sistema operacional, uso de CPU/GPU, compilação da UI para código de máquina e runtime compacto.
- A página indica runtime inferior a 300 KiB de RAM como característica declarada do runtime, não como consumo total do aplicativo.
- O projeto precisa validar separadamente acessibilidade, DPI e compatibilidade com Windows 10 durante a implementação; não se deve assumir que a descrição comercial substitui testes.

## Microsoft DPI
- A URL inicialmente consultada https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development retornou 404 em 14/08/2026.
- A afirmação sobre DPI awareness deve ser confirmada em uma página atual da documentação Microsoft antes de ser citada como requisito normativo.

## Repositório
- Repositório: danilo-jesus-unifil/Rovex.
- Estado inicial: somente LICENSE e README.md; README contém apenas “Testando o manus ilimitado”.
- Branch: main, sincronizada com origin/main no momento da clonagem.

## DPI awareness no Windows
- Fonte oficial atual: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process
- A Microsoft recomenda declarar o DPI padrão do processo no manifesto, em vez de definir isso por API quando possível.
- A documentação descreve `<dpiAwareness>` (Windows 10, versão 1607, com lista ordenada de modos e fallback) e `<dpiAware>` para versões anteriores.
- A implementação deve preferir configuração no manifesto e testar Per-Monitor DPI em Windows 10/11; não deve basear-se na URL antiga que retornou 404.

## Acessibilidade do Slint
- Fonte oficial atual: https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/
- A documentação recomenda declarar cedo propriedades de acessibilidade em componentes personalizados, incluindo papel, rótulo e ações.
- Acessibilidade precisa ser testada no Windows com ferramentas apropriadas; a documentação do framework não substitui testes reais com teclado e leitor de tela.
