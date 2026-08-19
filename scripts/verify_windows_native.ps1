$ErrorActionPreference = 'Stop'

$fixture = Join-Path $env:RUNNER_TEMP "rovex-native-cli-$PID"
try {
    New-Item -ItemType Directory -Path $fixture -Force | Out-Null
    $unicodeName = "arquivo ação seguro.txt"
    Set-Content -LiteralPath (Join-Path $fixture $unicodeName) -Value "Rovex native CLI" -NoNewline -Encoding utf8
    New-Item -ItemType Directory -Path (Join-Path $fixture "pasta com espaço") -Force | Out-Null

    $output = & cargo run --quiet -- --cli $fixture 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "CLI nativo retornou código $LASTEXITCODE`n$($output -join [Environment]::NewLine)"
    }
    $joined = $output -join [Environment]::NewLine
    if ($joined -notmatch 'Rovex core') {
        throw "saída do CLI não contém o cabeçalho esperado: $joined"
    }
    if ($joined -notmatch [regex]::Escape($unicodeName)) {
        throw "saída do CLI não preservou nome Unicode: $joined"
    }
    if ($joined -notmatch [regex]::Escape('pasta com espaço')) {
        throw "saída do CLI não preservou diretório com espaço: $joined"
    }

    Write-Host "Windows native CLI smoke passed: $fixture"
}
finally {
    if (Test-Path -LiteralPath $fixture) {
        Remove-Item -LiteralPath $fixture -Recurse -Force -ErrorAction SilentlyContinue
    }
}
