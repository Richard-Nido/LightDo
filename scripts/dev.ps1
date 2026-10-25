param([switch]$Isolated)
. "$PSScriptRoot/common.ps1"
if ($Isolated) { $env:LIGHTDO_DATA_DIR = Join-Path $script:ProjectRoot 'local-evidence\dev-data' }
Invoke-Checked (Find-Cargo) run --locked
