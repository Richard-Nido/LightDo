. "$PSScriptRoot/common.ps1"
$cargoExe = Find-Cargo
Invoke-Checked $cargoExe fmt --check
Invoke-Checked $cargoExe check --locked --all-targets
Invoke-Checked $cargoExe test --locked --lib
