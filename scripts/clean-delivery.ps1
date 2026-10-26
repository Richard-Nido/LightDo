param([switch]$Execute, [switch]$IncludeTestData)
. "$PSScriptRoot/common.ps1"
$workspace = [IO.Path]::GetFullPath($script:ProjectRoot).TrimEnd('\')
$prefix = $workspace + [IO.Path]::DirectorySeparatorChar
$gitRoot = (& git rev-parse --show-toplevel)
if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($gitRoot).TrimEnd('\') -ne $workspace) {
    throw '只能清理脚本所属的 Git 项目根目录。'
}
if (!(Test-Path -LiteralPath (Join-Path $workspace 'Cargo.toml'))) { throw '项目标识缺失。' }
$targets = @('target', '.tools', '.reference', 'local-evidence', 'dist/stage') | ForEach-Object {
    $resolved = [IO.Path]::GetFullPath((Join-Path $workspace $_))
    if (!$resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw '清理目标越过工作目录。' }
    if (Test-Path -LiteralPath $resolved) { $resolved }
}
# Do every guard before removing anything. Never follow directory junctions.
foreach ($target in $targets) {
    $relative = [IO.Path]::GetRelativePath($workspace, $target).Replace('\','/')
    $tracked = @(& git ls-files -- $relative)
    if ($LASTEXITCODE -ne 0 -or $tracked.Count) { throw "清理目标含已跟踪文件：$relative" }
    $links = @(
        Get-Item -LiteralPath $target -Force | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }
        Get-ChildItem -LiteralPath $target -Recurse -Force -Attributes ReparsePoint
    )
    if ($links.Count) { throw "清理目标含目录链接，未删除任何文件：$relative" }
}
$running = @(Get-Process lightdo -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -and $_.Path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)
})
$testStates = @(Get-ChildItem -LiteralPath (Join-Path $workspace 'local-evidence') -Filter state.json -Recurse -File -ErrorAction SilentlyContinue)
$plan = [ordered]@{
    execute = [bool]$Execute
    relativeTargets = @($targets | ForEach-Object { [IO.Path]::GetRelativePath($workspace, $_) })
    runningWorkspaceApp = ($running.Count -gt 0)
    isolatedStateFiles = $testStates.Count
    retained = @('source and Git history', 'docs and licenses', 'dist installers, license archives and checksums')
}
$plan | ConvertTo-Json -Depth 3
if (!$Execute) { return }
if ((& git status --porcelain).Length) { throw '工作区有未提交修改；未执行清理。' }
if ($running.Count) { throw '本目录的轻待仍在运行。请正常退出或使用已安装版本；不强杀。' }
if ($testStates.Count -and !$IncludeTestData) {
    throw '存在隔离验收数据。确认它们仅为测试数据或已妥善保留后，显式指定 -IncludeTestData。'
}
foreach ($target in $targets) { Remove-Item -LiteralPath $target -Recurse -Force }
Write-Output '中间目录已清理；源码、文档及 dist 正式交付文件保留。'
