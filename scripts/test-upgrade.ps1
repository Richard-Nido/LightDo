param(
    [Parameter(Mandatory)][string]$PreviousInstaller,
    [Parameter(Mandatory)][string]$Installer,
    [Parameter(Mandatory)][string]$SeedState,
    [switch]$RetainInstalled
)
. "$PSScriptRoot/common.ps1"
$PreviousInstaller = (Resolve-Path -LiteralPath $PreviousInstaller).Path
$Installer = (Resolve-Path -LiteralPath $Installer).Path
$SeedState = (Resolve-Path -LiteralPath $SeedState).Path
$installDir = Join-Path $env:LOCALAPPDATA 'Programs/LightDo'
$dataDir = Join-Path $env:APPDATA 'LightDo'
$stateFile = Join-Path $dataDir 'state.json'
$menuDir = Join-Path ([Environment]::GetFolderPath('Programs')) 'LightDo'
$appKey = 'HKCU:\Software\LightDo'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo'
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
foreach ($target in @($installDir,$dataDir,$menuDir,$appKey,$uninstallKey)) {
    if (Test-Path -LiteralPath $target) { throw '已有安装或数据，升级验收未执行，不覆盖既有内容。' }
}
if (Get-ItemProperty -LiteralPath $runKey -Name LightDo -ErrorAction SilentlyContinue) { throw '已有自启登记，未执行。' }
if (Get-Process lightdo -ErrorAction SilentlyContinue) { throw '轻待仍运行，请先正常保存退出；不强杀。' }
$seed = Get-Content -LiteralPath $SeedState -Raw | ConvertFrom-Json
if ($seed.version -ne 1 -or $null -eq $seed.todos) { throw '种子文件不是当前轻待数据格式。' }
function Install-Package([string]$Path) {
    $process = Start-Process -FilePath $Path -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "安装失败：$($process.ExitCode)。保留现场。" }
}
Install-Package $PreviousInstaller
$previousVersion = (Get-ItemProperty -LiteralPath $uninstallKey).DisplayVersion
$previousExeHash = (Get-FileHash -LiteralPath (Join-Path $installDir 'lightdo.exe')).Hash
New-Item -ItemType Directory -Path $dataDir | Out-Null
Copy-Item -LiteralPath $SeedState -Destination $stateFile
$seedHash = (Get-FileHash -LiteralPath $stateFile).Hash
Install-Package $Installer
$registration = Get-ItemProperty -LiteralPath $uninstallKey
$newExe = Join-Path $installDir 'lightdo.exe'
$newVersion = (Get-Item -LiteralPath $newExe).VersionInfo.ProductVersion
if ([version]$newVersion -le [version]$previousVersion) { throw '没有发生跨版本升级。保留现场。' }
if ($registration.DisplayVersion -ne $newVersion -or $registration.InstallLocation -ne $installDir) { throw '升级登记不一致。' }
if ((Get-FileHash -LiteralPath $newExe).Hash -ne (Get-FileHash -LiteralPath 'dist/stage/lightdo.exe').Hash) { throw '新 payload 与打包源不一致。' }
if ((Get-FileHash -LiteralPath $stateFile).Hash -ne $seedHash) { throw '升级改动了原任务或设置文件。保留现场。' }
$result = [ordered]@{
    fromVersion = $previousVersion
    toVersion = $newVersion
    crossVersionUpgrade = $true
    tasksAndSettingsByteIdentical = $true
    taskCount = $seed.todos.Count
    payloadMatches = $true
    executableChanged = ($previousExeHash -ne (Get-FileHash -LiteralPath $newExe).Hash)
    normalUserInstall = $true
    retainedForGuiAcceptance = [bool]$RetainInstalled
    guiObserved = $false
}
if (!$RetainInstalled) {
    # Retain every copied task. This test never deletes an application profile.
    $process = Start-Process -FilePath (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw '卸载失败，保留现场。' }
    $deadline = (Get-Date).AddSeconds(10)
    while ((Test-Path -LiteralPath $newExe) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 100 }
    if ((Test-Path -LiteralPath $newExe) -or (Test-Path -LiteralPath $uninstallKey)) { throw '卸载尚未完成。' }
    if ((Get-FileHash -LiteralPath $stateFile).Hash -ne $seedHash) { throw '默认卸载没有保留数据。' }
    $result.defaultUninstallRetainsData = $true
}
New-Item -ItemType Directory -Force 'local-evidence' | Out-Null
$result | ConvertTo-Json | Tee-Object -FilePath 'local-evidence/installer-upgrade.json'
