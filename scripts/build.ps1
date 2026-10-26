. "$PSScriptRoot/common.ps1"
Invoke-Checked (Find-Cargo) build --locked --release --target x86_64-pc-windows-msvc
Write-Output (Get-ReleaseBinary)
