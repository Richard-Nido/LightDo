param([switch]$Execute, [switch]$IncludeTestData)
. "$PSScriptRoot/common.ps1"
$workspace = [IO.Path]::GetFullPath($script:ProjectRoot).TrimEnd('\')
$prefix = $workspace + [IO.Path]::DirectorySeparatorChar
$gitRoot = (& git rev-parse --show-toplevel)
if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($gitRoot).TrimEnd('\') -ne $workspace) {
    throw '只能清理脚本所属的 Git 项目根目录。'
}
if (!(Test-Path -LiteralPath (Join-Path $workspace 'Cargo.toml'))) { throw '项目标识缺失。' }
if ((Get-Item -LiteralPath $workspace).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw '项目根目录为链接，未执行清理。' }
$version = [regex]::Match((Get-Content -Raw (Join-Path $workspace 'Cargo.toml')), '(?m)^version = "([0-9]+\.[0-9]+\.[0-9]+)"\r?$').Groups[1].Value
if (!$version) { throw '无法确定应保留的交付版本。' }
$dist = Join-Path $workspace 'dist'
if ((Test-Path -LiteralPath $dist) -and ((Get-Item -LiteralPath $dist).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw '交付目录为链接，未执行清理。'
}
$obsoleteAssets = @(Get-ChildItem -LiteralPath $dist -File -ErrorAction SilentlyContinue | Where-Object {
    $_.Name -match '^LightDo_([0-9]+\.[0-9]+\.[0-9]+)_(x64-setup\.exe|licenses\.zip)$' -and $Matches[1] -ne $version
})
$targets = @('target', '.tools', '.reference', 'local-evidence', 'dist/stage') | ForEach-Object {
    $resolved = [IO.Path]::GetFullPath((Join-Path $workspace $_))
    if (!$resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw '清理目标越过工作目录。' }
    if (Test-Path -LiteralPath $resolved) { $resolved }
}
# Do every guard before removing anything. Never follow directory junctions.
foreach ($target in @($targets) + @($obsoleteAssets.FullName)) {
    $absolute = [IO.Path]::GetFullPath($target)
    if (!$absolute.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw '清理目标越过工作目录。' }
    $parent = Split-Path -Parent $absolute
    while ($parent -ne $workspace) {
        if (!$parent.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase) -or
            ((Get-Item -LiteralPath $parent).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw '清理目标的父目录为链接或越界，未删除任何文件。'
        }
        $parent = Split-Path -Parent $parent
    }
    $relative = [IO.Path]::GetRelativePath($workspace, $target).Replace('\','/')
    $tracked = @(& git ls-files -- $relative)
    if ($LASTEXITCODE -ne 0 -or $tracked.Count) { throw "清理目标含已跟踪文件：$relative" }
    $item = Get-Item -LiteralPath $target -Force
    $links = @(
        $item | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }
        if ($item.PSIsContainer) {
            Get-ChildItem -LiteralPath $target -Recurse -Force -Attributes ReparsePoint
        }
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
    obsoleteInstallersAndLicenses = @($obsoleteAssets.Name)
    retainedVersion = $version
    runningWorkspaceApp = ($running.Count -gt 0)
    isolatedStateFiles = $testStates.Count
    retained = @('source and Git history', 'docs and licenses', 'current dist installer, license archive and checksums')
}
$plan | ConvertTo-Json -Depth 3
if (!$Execute) { return }
if ((& git status --porcelain).Length) { throw '工作区有未提交修改；未执行清理。' }
if ($running.Count) { throw '本目录的轻待仍在运行。请正常退出或使用已安装版本；不强杀。' }
if ($testStates.Count -and !$IncludeTestData) {
    throw '存在隔离验收数据。确认它们仅为测试数据或已妥善保留后，显式指定 -IncludeTestData。'
}
foreach ($target in $targets) { Remove-Item -LiteralPath $target -Recurse -Force }
foreach ($asset in $obsoleteAssets) { Remove-Item -LiteralPath $asset.FullName -Force }
Write-Output '中间目录已清理；源码、文档及 dist 正式交付文件保留。'
