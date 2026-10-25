. "$PSScriptRoot/common.ps1"
$zip = Join-Path $script:ProjectRoot '.tools/nsis-3.12.zip'
$expected = '56581f90db321581c5381193d796fffcf2d24b2f8fed2160a6c6a3baa67f2c4f'
New-Item -ItemType Directory -Force '.tools' | Out-Null
if (!(Test-Path -LiteralPath $zip) -or (Get-FileHash -LiteralPath $zip).Hash.ToLowerInvariant() -ne $expected) {
    Invoke-WebRequest 'https://netix.dl.sourceforge.net/project/nsis/NSIS%203/3.12/nsis-3.12.zip' -OutFile $zip
}
if ((Get-FileHash -LiteralPath $zip).Hash.ToLowerInvariant() -ne $expected) { throw 'NSIS 官方包 SHA-256 校验失败，停止执行' }
Expand-Archive -LiteralPath $zip -DestinationPath '.tools/nsis' -Force
Write-Output (Join-Path $script:ProjectRoot '.tools/nsis/nsis-3.12/makensis.exe')
