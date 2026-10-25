. "$PSScriptRoot/common.ps1"
Invoke-Checked (Find-Cargo) build --locked --release --target x86_64-pc-windows-msvc
Write-Output (Join-Path $script:ProjectRoot 'target\x86_64-pc-windows-msvc\release\lightdo.exe')
