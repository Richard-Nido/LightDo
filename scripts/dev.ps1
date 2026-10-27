param([switch]$Isolated)
. "$PSScriptRoot/common.ps1"
$previousDataDirectory = $env:LIGHTDO_DATA_DIR
try {
    if ($Isolated) { $env:LIGHTDO_DATA_DIR = Join-Path $script:ProjectRoot 'local-evidence\dev-data' }
    Invoke-Checked (Find-Cargo) run --locked
} finally {
    # Do not leak an isolated test profile into later installed-app launches.
    $env:LIGHTDO_DATA_DIR = $previousDataDirectory
}
