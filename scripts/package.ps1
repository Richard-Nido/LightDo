param([string]$NsisPath, [switch]$SkipBuild)
. "$PSScriptRoot/common.ps1"
if (!$SkipBuild) { & "$PSScriptRoot/build.ps1"; if ($LASTEXITCODE -ne 0) { throw '生产构建失败' } }
$cargoExe = Find-Cargo
$metadataText = & $cargoExe metadata --locked --format-version 1
if ($LASTEXITCODE -ne 0) { throw '读取锁定依赖失败' }
$metadata = $metadataText | ConvertFrom-Json
$package = $metadata.packages | Where-Object name -eq 'lightdo'
$version = $package.version
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw '安装器要求三段版本号' }
$stage = Join-Path $script:ProjectRoot 'dist/stage'
New-Item -ItemType Directory -Force -Path $stage | Out-Null
$licenseDir=Join-Path $stage 'licenses'
New-Item -ItemType Directory -Force -Path $licenseDir | Out-Null
$binary = Get-ReleaseBinary
if (!(Test-Path -LiteralPath $binary)) { throw '找不到当前构建目录的 lightdo.exe。' }
if ((Get-Item -LiteralPath $binary).VersionInfo.ProductVersion -ne $version) {
    throw '待打包 EXE 的资源版本与源码版本不一致，禁止打包旧二进制。'
}
Copy-Item -LiteralPath $binary -Destination $stage
Copy-Item 'LICENSE','THIRD_PARTY_NOTICES.md','docs/LICENSE-Glance','docs/LICENSE-QuietDo' $stage
$licenses = @('LightDo locked dependency license inventory','')
foreach ($dependency in $metadata.packages | Sort-Object name,version) {
    if ($dependency.name -eq 'lightdo') { continue }
    if (!$dependency.license) { throw "依赖缺少许可：$($dependency.name)" }
    $licenses += "$($dependency.name) $($dependency.version) | $($dependency.license) | $($dependency.repository)"
    $crateDir=Split-Path -Parent $dependency.manifest_path
    $licenseFiles=@(Get-ChildItem -LiteralPath $crateDir -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE)' })
    if ($licenseFiles.Count -eq 0) { throw "依赖缺少随附许可证正文：$($dependency.name)" }
    foreach ($file in $licenseFiles) { Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $licenseDir "$($dependency.name)-$($dependency.version)--$($file.Name)") }
}
$licenses | Set-Content -Encoding utf8 (Join-Path $stage 'THIRD_PARTY_DEPENDENCIES.txt')
if (!$NsisPath) {
    $candidate = Get-Command makensis -ErrorAction SilentlyContinue
    if ($candidate) { $NsisPath = $candidate.Source }
    elseif (Test-Path '.tools/nsis/nsis-3.12/makensis.exe') { $NsisPath = Join-Path $script:ProjectRoot '.tools/nsis/nsis-3.12/makensis.exe' }
    elseif (Test-Path "${env:ProgramFiles(x86)}/NSIS/makensis.exe") { $NsisPath = "${env:ProgramFiles(x86)}/NSIS/makensis.exe" }
    else { throw '缺少 NSIS；请指定 -NsisPath。文档中提供官方来源。' }
}
$output = Join-Path $script:ProjectRoot "dist/LightDo_${version}_x64-setup.exe"
Invoke-Checked $NsisPath '/WX' '/INPUTCHARSET' 'UTF8' "/DVERSION=$version" "/DSTAGE=$stage" "/DOUTPUT=$output" 'installer/lightdo.nsi'
$licenseZip=Join-Path $script:ProjectRoot "dist/LightDo_${version}_licenses.zip"
Compress-Archive -Path "$stage/LICENSE","$stage/THIRD_PARTY_NOTICES.md","$stage/THIRD_PARTY_DEPENDENCIES.txt",$licenseDir -DestinationPath $licenseZip -Force
$sums=foreach ($asset in @($output,$licenseZip)) {$hash=(Get-FileHash -LiteralPath $asset -Algorithm SHA256).Hash.ToLowerInvariant();"$hash  $([IO.Path]::GetFileName($asset))"}
$sums | Set-Content -Encoding ascii 'dist/SHA256SUMS.txt'
Write-Output $output
