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
function Get-ReleaseBinary {
    $buildRoot = if ($env:CARGO_TARGET_DIR) {
        [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR, $script:ProjectRoot)
    } else {
        Join-Path $script:ProjectRoot 'target'
    }
    Join-Path $buildRoot 'x86_64-pc-windows-msvc/release/lightdo.exe'
}
function Invoke-Checked {
    # Keep this a simple function: advanced-function common parameters would
    # consume native flags such as cargo -p and git tag -a before invocation.
    param([string]$Executable)
    if (!$Executable) { throw '缺少待执行的程序路径。' }
    & $Executable @args
    if ($LASTEXITCODE -ne 0) {
        $failedCode = $LASTEXITCODE
        [Console]::Error.WriteLine("命令失败，退出码 ${failedCode}：$Executable")
        exit $failedCode
    }
}
Set-Location -LiteralPath $script:ProjectRoot
