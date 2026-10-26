param(
    [Parameter(Mandatory)][ValidatePattern('^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')][string]$Tag,
    [Parameter(Mandatory)][string]$AssetsDirectory,
    [switch]$ValidateOnly
)
. "$PSScriptRoot/common.ps1"
$assetRoot = (Resolve-Path -LiteralPath $AssetsDirectory).Path
$version = $Tag.Substring(1)
$sourceVersion = [regex]::Match((Get-Content -Raw 'Cargo.toml'), '(?m)^version = "([^"]+)"$').Groups[1].Value
if ($version -ne $sourceVersion) { throw '发布标签与源码版本不一致。' }
$expected = @("LightDo_${version}_x64-setup.exe", "LightDo_${version}_licenses.zip", 'SHA256SUMS.txt',
    'THIRD_PARTY_DEPENDENCIES.txt', 'LICENSE', 'THIRD_PARTY_NOTICES.md')
$files = @(Get-ChildItem -LiteralPath $assetRoot -Recurse -File)
$names = @($files.Name)
if ($files.Count -ne $expected.Count -or @($names | Select-Object -Unique).Count -ne $files.Count -or
    @(Compare-Object $expected $names).Count) { throw '发布资产缺失、重复或版本不匹配。' }
$hashes = @{}
foreach ($file in $files) { $hashes[$file.Name] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
$checksumFile = ($files | Where-Object Name -eq 'SHA256SUMS.txt').FullName
$checked = @()
foreach ($line in Get-Content -LiteralPath $checksumFile) {
    if ($line -notmatch '^([a-fA-F0-9]{64})  ([^/\\]+)$') { throw '校验文件格式无效。' }
    $name = $Matches[2]
    if ($name -notin $expected[0..1] -or $name -in $checked -or $hashes[$name] -ne $Matches[1].ToLowerInvariant()) {
        throw '安装包或许可压缩包的 SHA-256 不匹配。'
    }
    $checked += $name
}
if ($checked.Count -ne 2) { throw '校验文件没有覆盖安装包和许可压缩包。' }
if ($ValidateOnly) { Write-Output "Release assets verified: $Tag ($($files.Count) files)"; return }

# Only the publish job receives contents:write. Failed uploads stay private
# as a draft; no public release is exposed until every downloaded asset matches.
$paths = @($files.FullName)
Invoke-Checked gh release create $Tag @paths --draft --verify-tag --title "轻待 LightDo $Tag" --notes-file docs/release-notes.md
$verifyRoot = Join-Path $script:ProjectRoot ('target/release-verify-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $verifyRoot | Out-Null
Invoke-Checked gh release download $Tag --dir $verifyRoot
$downloaded = @(Get-ChildItem -LiteralPath $verifyRoot -File)
if (@(Compare-Object $expected @($downloaded.Name)).Count) { throw '草稿 Release 下载资产不完整，保留草稿及本地证据。' }
foreach ($file in $downloaded) {
    if ((Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hashes[$file.Name]) {
        throw '草稿 Release 下载校验失败，未公开。'
    }
}
Invoke-Checked gh release edit $Tag --draft=false --prerelease=false --latest --verify-tag
$publishedText = & gh release view $Tag --json isDraft,isPrerelease,tagName,url
if ($LASTEXITCODE -ne 0) { throw '无法确认公开 Release 状态。' }
$published = $publishedText | ConvertFrom-Json
if ($published.isDraft -ne $false -or $published.isPrerelease -ne $false -or $published.tagName -ne $Tag) {
    throw 'Release 未达到公开正式版本状态。'
}
# Delete only the exact fresh verification directory inside this workspace.
$verifiedPath = [IO.Path]::GetFullPath($verifyRoot)
$targetPrefix = [IO.Path]::GetFullPath((Join-Path $script:ProjectRoot 'target')) + [IO.Path]::DirectorySeparatorChar
if (!$verifiedPath.StartsWith($targetPrefix, [StringComparison]::OrdinalIgnoreCase) -or
    (Get-Item -LiteralPath $verifiedPath).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw '校验目录越界，未清理。' }
Remove-Item -LiteralPath $verifiedPath -Recurse -Force
Write-Output $published.url
