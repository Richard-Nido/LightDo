param(
    [ValidateRange(10,180)][int]$Seconds = 30,
    [ValidatePattern('^[a-z0-9-]{1,64}$')][string]$Name = 'runtime',
    [switch]$IncludeGpu,
    [int]$ProcessId = 0
)
. "$PSScriptRoot/common.ps1"
if ($ProcessId) { $taskProcess = Get-Process -Id $ProcessId -ErrorAction Stop }
else {
    $taskProcesses = @(Get-Process lightdo -ErrorAction Stop)
    if ($taskProcesses.Count -ne 1) { throw '必须只有一个轻待测试实例，或显式指定 ProcessId。' }
    $taskProcess = $taskProcesses[0]
}
if ($taskProcess.ProcessName -ne 'lightdo') { throw '只测量轻待进程。' }
$taskLogical = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
$taskInitial = & $taskProcess.Path --diagnose | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw '读取本应用诊断失败。' }
$taskCpuStart = $taskProcess.TotalProcessorTime.TotalMilliseconds
$taskWatch = [Diagnostics.Stopwatch]::StartNew()
$taskMemory = [Collections.Generic.List[double]]::new()
$taskGpu = [Collections.Generic.List[double]]::new()
$taskGpuDedicated = [Collections.Generic.List[double]]::new()
$taskGpuShared = [Collections.Generic.List[double]]::new()
$taskGpuError = $null
function Add-MemorySample {
    $taskProcess.Refresh()
    if ($taskProcess.HasExited) { throw '测试进程已退出，本轮测量无效。' }
    $taskMemory.Add($taskProcess.WorkingSet64)
}
Add-MemorySample
if ($IncludeGpu) {
    try {
        Get-Counter -Counter '\GPU Engine(*)\Utilization Percentage',
            '\GPU Process Memory(*)\Dedicated Usage', '\GPU Process Memory(*)\Shared Usage' `
            -SampleInterval 1 -MaxSamples ($Seconds + 1) -ErrorAction Stop | ForEach-Object {
                Add-MemorySample
                $taskSamples = @($_.CounterSamples | Where-Object {
                    $_.InstanceName -match "pid_$($taskProcess.Id)_" -and $_.Status -in 0,1
                })
                $taskEngines = @($taskSamples | Where-Object Path -like '*utilization percentage')
                if ($taskEngines.Count) {
                    $taskGpu.Add(($taskEngines.CookedValue | Measure-Object -Maximum).Maximum)
                }
                foreach ($taskMetric in @('Dedicated Usage','Shared Usage')) {
                    $taskValues = @($taskSamples | Where-Object Path -like "*$taskMetric")
                    if ($taskValues.Count) {
                        $taskBytes = ($taskValues.CookedValue | Measure-Object -Sum).Sum
                        if ($taskMetric -eq 'Dedicated Usage') { $taskGpuDedicated.Add($taskBytes) }
                        else { $taskGpuShared.Add($taskBytes) }
                    }
                }
            }
    } catch { $taskGpuError = $_.Exception.Message }
}
while ($taskWatch.Elapsed.TotalSeconds -lt $Seconds) {
    Start-Sleep -Milliseconds 1000
    Add-MemorySample
}
$taskProcess.Refresh()
$taskElapsed = $taskWatch.Elapsed.TotalSeconds
$taskCpu = $taskProcess.TotalProcessorTime.TotalMilliseconds - $taskCpuStart
$taskFinal = & $taskProcess.Path --diagnose | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw '测量后的本应用诊断失败。' }
$taskResult = [ordered]@{
    name = $Name
    measuredAt = [DateTimeOffset]::Now.ToString('o')
    seconds = $taskElapsed
    logicalProcessors = $taskLogical
    cpuMilliseconds = $taskCpu
    cpuPercentNormalized = 100 * $taskCpu / (1000 * $taskElapsed * $taskLogical)
    workingSetMeanMiB = ($taskMemory | Measure-Object -Average).Average / 1MB
    workingSetEndMiB = $taskProcess.WorkingSet64 / 1MB
    privateMiB = $taskProcess.PrivateMemorySize64 / 1MB
    handles = $taskProcess.HandleCount
    gpuMetric = 'Maximum active engine utilization of this process per sample, not total system GPU'
    gpuSamples = $taskGpu.Count
    gpuMeanPercent = if ($taskGpu.Count) { ($taskGpu | Measure-Object -Average).Average } else { $null }
    gpuPeakPercent = if ($taskGpu.Count) { ($taskGpu | Measure-Object -Maximum).Maximum } else { $null }
    dedicatedGpuMeanMiB = if ($taskGpuDedicated.Count) { ($taskGpuDedicated | Measure-Object -Average).Average / 1MB } else { $null }
    sharedGpuMeanMiB = if ($taskGpuShared.Count) { ($taskGpuShared | Measure-Object -Average).Average / 1MB } else { $null }
    gpuError = $taskGpuError
    initial = $taskInitial
    final = $taskFinal
    binarySha256 = (Get-FileHash -LiteralPath $taskProcess.Path -Algorithm SHA256).Hash.ToLowerInvariant()
}
$taskOutput = Join-Path $script:ProjectRoot "local-evidence/perf-$Name.json"
New-Item -ItemType Directory -Force (Split-Path -Parent $taskOutput) | Out-Null
$taskResult | ConvertTo-Json -Depth 8 | Tee-Object -FilePath $taskOutput
