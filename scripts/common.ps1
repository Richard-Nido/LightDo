$ErrorActionPreference = 'Stop'
$script:ProjectRoot = Split-Path -Parent $PSScriptRoot
function Find-Cargo {
    $found = Get-Command cargo -ErrorAction SilentlyContinue
    if ($found) { return $found.Source }
    $candidate = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path -LiteralPath $candidate) { return $candidate }
    throw '缺少 Rust MSVC 工具链。请安装 rustup，然后重新运行。'
}
function Invoke-Checked {
    param([Parameter(Mandatory)][string]$Executable, [Parameter(ValueFromRemainingArguments)][string[]]$Arguments)
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) { throw "命令失败，退出码 ${LASTEXITCODE}：$Executable" }
}
Set-Location -LiteralPath $script:ProjectRoot
