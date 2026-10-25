. "$PSScriptRoot/common.ps1"
$zip = Join-Path $script:ProjectRoot '.tools/nsis-3.12.zip'
$expected = '56581f90db321581c5381193d796fffcf2d24b2f8fed2160a6c6a3baa67f2c4f'
New-Item -ItemType Directory -Force '.tools' | Out-Null
if (!(Test-Path -LiteralPath $zip) -or (Get-FileHash -LiteralPath $zip).Hash.ToLowerInvariant() -ne $expected) {
    $download=Join-Path $script:ProjectRoot '.tools/nsis-download.tmp'
    $verified=$false
    foreach ($mirror in @('phoenixnap','pilotfiber','master','netix')) {
        $url="https://$mirror.dl.sourceforge.net/project/nsis/NSIS%203/3.12/nsis-3.12.zip"
        & curl.exe --fail --location --max-time 30 --proto '=https' --proto-redir '=https' --silent --show-error $url -o $download
        if ($LASTEXITCODE -eq 0 -and (Test-Path -LiteralPath $download) -and (Get-FileHash -LiteralPath $download).Hash.ToLowerInvariant() -eq $expected) {
            Move-Item -LiteralPath $download -Destination $zip -Force
            $verified=$true
            break
        }
        Write-Host "官方镜像 $mirror 未返回固定哈希的安装工具包，尝试下一个镜像。"
    }
    if (!$verified) { throw '所有官方镜像均未返回通过固定 SHA-256 校验的 NSIS，停止执行' }
}
if ((Get-FileHash -LiteralPath $zip).Hash.ToLowerInvariant() -ne $expected) { throw 'NSIS 官方包 SHA-256 校验失败，停止执行' }
Expand-Archive -LiteralPath $zip -DestinationPath '.tools/nsis' -Force
Write-Output (Join-Path $script:ProjectRoot '.tools/nsis/nsis-3.12/makensis.exe')
