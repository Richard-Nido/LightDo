. "$PSScriptRoot/common.ps1"
foreach ($scriptFile in Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.ps1' -File) {
    $scriptTokens = $null
    $scriptErrors = $null
    [Management.Automation.Language.Parser]::ParseFile($scriptFile.FullName, [ref]$scriptTokens, [ref]$scriptErrors) | Out-Null
    if ($scriptErrors.Count) { throw "PowerShell语法检查失败：$($scriptFile.Name)：$($scriptErrors[0].Message)" }
}
& "$PSScriptRoot/test-clean-delivery.ps1"
if ($LASTEXITCODE -ne 0) { throw '隔离清理验收失败。' }
$cargoExe = Find-Cargo
Invoke-Checked $cargoExe fmt --check
Invoke-Checked $cargoExe check --locked --all-targets
Invoke-Checked $cargoExe test --locked --lib
