#requires -Version 7.4
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
    if ($LASTEXITCODE -ne 0) {
        $failedCode = $LASTEXITCODE
        [Console]::Error.WriteLine("命令失败，退出码 ${failedCode}：$Executable")
        exit $failedCode
    }
}
Set-Location -LiteralPath $script:ProjectRoot
