param([string]$Installer)
. "$PSScriptRoot/common.ps1"
if (!$Installer) {
    $metadata=& (Find-Cargo) metadata --locked --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) { throw '读取版本失败' }
    $version=(($metadata|ConvertFrom-Json).packages|Where-Object name -eq 'lightdo').version
    $Installer=Join-Path $script:ProjectRoot "dist/LightDo_${version}_x64-setup.exe"
}
$Installer=(Resolve-Path -LiteralPath $Installer).Path
$installDir=Join-Path $script:ProjectRoot '.tools/install-smoke'
$dataDir=Join-Path $env:APPDATA 'LightDo'
$menuDir=Join-Path ([Environment]::GetFolderPath('Programs')) 'LightDo'
$uninstallKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo'
$appKey='HKCU:\Software\LightDo'
$runKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
# Existing application data and registrations are never overwritten by a
# test. Use an empty profile/Sandbox if any of these guards fire.
if ((Test-Path -LiteralPath $uninstallKey) -or (Test-Path -LiteralPath $appKey) -or (Test-Path -LiteralPath $dataDir) -or (Test-Path -LiteralPath $menuDir) -or (Test-Path -LiteralPath $installDir)) {
    throw '已存在 LightDo 安装、数据或测试目录。未更改任何内容；请使用干净用户配置进行安装冒烟测试。'
}
if (Get-ItemProperty -LiteralPath $runKey -Name LightDo -ErrorAction SilentlyContinue) { throw '已存在 LightDo 自启注册，测试停止，未更改该注册。' }
New-Item -ItemType Directory -Path $dataDir | Out-Null
$marker=Join-Path $dataDir '.installer-smoke-marker'
'LightDo synthetic installer retention marker' | Set-Content -Encoding utf8 -LiteralPath $marker
$markerHash=(Get-FileHash -LiteralPath $marker).Hash
function Run-Installer {
    $process=Start-Process -FilePath $Installer -ArgumentList "/S /D=$installDir" -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "安装失败，退出码 $($process.ExitCode)；保留现场，不删除任何应用数据。" }
}
Run-Installer
$installedExe=Join-Path $installDir 'lightdo.exe'
if (!(Test-Path -LiteralPath $installedExe)) { throw '安装后缺少 lightdo.exe' }
if ((Get-FileHash -LiteralPath $installedExe).Hash -ne (Get-FileHash 'dist/stage/lightdo.exe').Hash) { throw '安装后的 EXE 与打包源不一致' }
$registration=Get-ItemProperty -LiteralPath $uninstallKey
if ($registration.InstallLocation -ne $installDir -or $registration.Publisher -ne 'Yuan Haoran (Richard-Nido)') { throw '卸载登记路径或发布者不一致' }
$payloadVersion=(Get-Item -LiteralPath $installedExe).VersionInfo.ProductVersion
if ($registration.DisplayVersion -ne $payloadVersion) { throw '应用资源版本与安装器登记版本不一致' }
if (!$version) { $version=$payloadVersion }
if ($registration.UninstallString -ne ('"'+(Join-Path $installDir 'uninstall.exe')+'"')) { throw '卸载命令引号不正确' }
if (!(Test-Path -LiteralPath (Join-Path $menuDir '轻待 LightDo.lnk'))) { throw '缺少当前用户开始菜单入口' }
Run-Installer
if ((Get-FileHash -LiteralPath $marker).Hash -ne $markerHash) { throw '同版本覆盖安装改动了用户数据目录标记' }
$process=Start-Process -FilePath (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "卸载失败：$($process.ExitCode)" }
$deadline=(Get-Date).AddSeconds(10)
while (((Test-Path -LiteralPath $uninstallKey) -or (Test-Path -LiteralPath $installedExe)) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 100 }
if ((Test-Path -LiteralPath $uninstallKey) -or (Test-Path -LiteralPath $installedExe)) { throw '卸载没有完成；保留现场' }
if (!(Test-Path -LiteralPath $marker) -or (Get-FileHash -LiteralPath $marker).Hash -ne $markerHash) { throw '默认卸载没有保留数据目录标记' }
if (Test-Path -LiteralPath $menuDir) { throw '开始菜单入口没有清理' }
if (Get-ItemProperty -LiteralPath $runKey -Name LightDo -ErrorAction SilentlyContinue) { throw '自启项没有清理' }
# Delete only this script's own synthetic marker, never pre-existing data.
Remove-Item -LiteralPath $marker
if (@(Get-ChildItem -LiteralPath $dataDir -Force).Count -eq 0) { Remove-Item -LiteralPath $dataDir }
$result=[ordered]@{version=$version;silentInstall=$true;payloadHashMatches=$true;sameVersionOverwrite=$true;uninstallRegistration=$true;defaultDataRetention=$true;shortcutsCleaned=$true;guiObserved=$false;crossVersionUpgradeTested=$false;activeDeleteDataChoiceTested=$false}
New-Item -ItemType Directory -Force 'local-evidence' | Out-Null
$result|ConvertTo-Json|Tee-Object -FilePath 'local-evidence/installer-smoke.json'
