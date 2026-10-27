param(
    [Parameter(Mandatory)][string]$Executable,
    [ValidateRange(2,10)][int]$Cycles = 3
)
. "$PSScriptRoot/common.ps1"
$Executable = (Resolve-Path -LiteralPath $Executable).Path
if ([IO.Path]::GetFileName($Executable) -ne 'lightdo.exe') { throw '仅验收轻待应用。' }
if ($env:LIGHTDO_DATA_DIR) { throw '请先移除开发数据目录覆盖，避免把隔离数据当正式数据。' }
$statePath = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'LightDo/state.json'
$model = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
if ($model.version -ne 1 -or !$model.todos.Count) { throw '需要现有非空轻待清单，未修改文件。' }
$expectedTasks = $model.todos.Count
$expectedUnfinished = @($model.todos | Where-Object { !$_.done }).Count
$records = @()
function Stop-Normally {
    $running = @(Get-Process lightdo -ErrorAction SilentlyContinue)
    if ($running.Count -gt 1 -or ($running.Count -eq 1 -and $running[0].Path -ne $Executable)) {
        throw '有其他轻待实例，未操作。'
    }
    if (!$running.Count) { return }
    $shutdown = Start-Process -FilePath $Executable -ArgumentList '--shutdown' -WindowStyle Hidden -Wait -PassThru
    if ($shutdown.ExitCode -ne 0) { throw '正常退出请求失败。' }
    $deadline = (Get-Date).AddSeconds(8)
    while ((Get-Process -Id $running[0].Id -ErrorAction SilentlyContinue) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 100 }
    if (Get-Process -Id $running[0].Id -ErrorAction SilentlyContinue) { throw '未正常退出；不强杀，不继续。' }
}
Stop-Normally
$baselineHash = (Get-FileHash -LiteralPath $statePath).Hash
$baselineContent = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json | ConvertTo-Json -Depth 16 -Compress
function Assert-StateContent {
    $currentContent = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json | ConvertTo-Json -Depth 16 -Compress
    if ($currentContent -ne $baselineContent) { throw '正常退出改变了任务或设置内容；停止并保留现场。' }
}
for ($cycle=1; $cycle -le $Cycles; $cycle++) {
    # Cold starts from a different working directory must read the same folder.
    $startupWatch = [Diagnostics.Stopwatch]::StartNew()
    $process = Start-Process -FilePath $Executable -WorkingDirectory ([IO.Path]::GetTempPath()) -PassThru
    $deadline = (Get-Date).AddSeconds(12)
    $diagnostic = $null
    do {
        Start-Sleep -Milliseconds 200
        if ($process.HasExited) { throw "第${cycle}次启动提前退出。" }
        $diagnostic = & $Executable --diagnose | ConvertFrom-Json
        if ($LASTEXITCODE -ne 0) { throw '诊断进程失败，停止采样。' }
    } until ($null -ne $diagnostic.data -or (Get-Date) -ge $deadline)
    if ($diagnostic.data.readable -ne $true -or $diagnostic.data.tasks -ne $expectedTasks -or $diagnostic.data.unfinished -ne $expectedUnfinished) {
        throw "第${cycle}次读取结果不符；保留现场，不覆盖任务文件。"
    }
    $records += [pscustomobject]@{cycle=$cycle;pid=$process.Id;tasks=$diagnostic.data.tasks;unfinished=$diagnostic.data.unfinished;readable=$diagnostic.data.readable;dataAndInputReadyMilliseconds=$startupWatch.Elapsed.TotalMilliseconds}
    if ($cycle -lt $Cycles) {
        Stop-Normally
        Assert-StateContent
    }
}
Assert-StateContent
$result = [ordered]@{binarySha256=(Get-FileHash $Executable).Hash.ToLowerInvariant();cycles=$Cycles;separateProcesses=$records;startupTiming='Includes process launch, 200ms polling and diagnostic subprocess; data and native input created, not first visible glass frame';foreignWorkingDirectory=$true;stateContentUnchanged=$true;stateHashUnchanged=((Get-FileHash -LiteralPath $statePath).Hash -eq $baselineHash);formattingNote='Full parsed JSON content must remain equal; whitespace formatting is reported separately';taskTextLogged=$false;guiObserved=$false}
$result | ConvertTo-Json -Depth 5 | Tee-Object -FilePath (Join-Path $script:ProjectRoot 'local-evidence/restart-retention.json')
