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

    $longDirectory = $fixture
    for ($index = 0; $index -lt 4; $index++) {
        $segment = "segmento-$index-" + ('x' * 54)
        $longDirectory = Join-Path $longDirectory $segment
        New-Item -ItemType Directory -Path $longDirectory -Force | Out-Null
    }
    $longFile = Join-Path $longDirectory (('arquivo-' + ('y' * 40)) + '.txt')
    Set-Content -LiteralPath $longFile -Value "Rovex long path" -NoNewline -Encoding utf8
    if ($longFile.Length -le 260) {
        throw "fixture de caminho longo não ultrapassou MAX_PATH: $($longFile.Length)"
    }
    $longOutput = & cargo run --quiet -- --cli $longDirectory 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "CLI falhou no caminho longo ($($longFile.Length) caracteres): $($longOutput -join [Environment]::NewLine)"
    }
    $longJoined = $longOutput -join [Environment]::NewLine
    if ($longJoined -notmatch [regex]::Escape((Split-Path -Leaf $longFile))) {
        throw "saída do CLI não preservou o arquivo no caminho longo: $longJoined"
    }

    $junctionTarget = Join-Path $fixture "junction-target"
    New-Item -ItemType Directory -Path $junctionTarget -Force | Out-Null
    $junctionMarker = Join-Path $junctionTarget "outside-marker.txt"
    Set-Content -LiteralPath $junctionMarker -Value "must not be followed" -NoNewline -Encoding utf8
    $junction = Join-Path $fixture "junction-entry"
    New-Item -ItemType Junction -Path $junction -Target $junctionTarget -Force | Out-Null
    $junctionOutput = & cargo run --quiet -- --cli $junction 2>&1
    if ($LASTEXITCODE -eq 0) {
        throw "CLI seguiu junction em vez de recusar o ponto de reparse: $($junctionOutput -join [Environment]::NewLine)"
    }
    $junctionJoined = $junctionOutput -join [Environment]::NewLine
    if ($junctionJoined -notmatch 'reparse|redirecionado|inválido|não encontrado') {
        throw "CLI recusou junction sem diagnóstico controlado: $junctionJoined"
    }

    Write-Host "Windows native CLI smoke passed: $fixture; long path length $($longFile.Length); junction rejected"
}
finally {
    if (Test-Path -LiteralPath $fixture) {
        Remove-Item -LiteralPath $fixture -Recurse -Force -ErrorAction SilentlyContinue
    }
}
