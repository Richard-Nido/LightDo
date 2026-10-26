. "$PSScriptRoot/common.ps1"
$fixtureParent = [IO.Path]::GetFullPath((Join-Path $script:ProjectRoot 'local-evidence'))
$fixture = [IO.Path]::GetFullPath((Join-Path $fixtureParent ('cleanup-fixture-' + [guid]::NewGuid().ToString('N'))))
if (!$fixture.StartsWith($fixtureParent + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw '清理测试目录越界。'
}
$originalLocation = Get-Location
$cleanupSource = Join-Path $PSScriptRoot 'clean-delivery.ps1'
function Fixture-Git {
    & git -C $fixture @args
    if ($LASTEXITCODE -ne 0) { throw '隔离测试仓库命令失败。' }
}
function Expect-Refusal([string]$Message) {
    $rejected = $false
    try { & (Join-Path $fixture 'scripts/clean-delivery.ps1') -Execute -IncludeTestData | Out-Null }
    catch { if ($_.Exception.Message -notlike "*$Message*") { throw }; $rejected = $true }
    if (!$rejected) { throw '清理安全门未拒绝测试用例。' }
    if (!(Test-Path -LiteralPath (Join-Path $fixture 'target/sentinel.txt'))) { throw '拒绝前已经删除测试文件。' }
}
try {
    foreach ($relative in @('scripts','target','.tools','.reference','local-evidence','dist/stage')) {
        New-Item -ItemType Directory -Path (Join-Path $fixture $relative) -Force | Out-Null
    }
    Copy-Item -LiteralPath $cleanupSource,(Join-Path $PSScriptRoot 'common.ps1') -Destination (Join-Path $fixture 'scripts')
    @('[package]','name = "lightdo-cleanup-fixture"','version = "1.0.0"') | Set-Content -Encoding utf8 (Join-Path $fixture 'Cargo.toml')
    @('target/','.tools/','.reference/','local-evidence/','dist/') | Set-Content -Encoding utf8 (Join-Path $fixture '.gitignore')
    'synthetic disposable file' | Set-Content -Encoding utf8 (Join-Path $fixture 'target/sentinel.txt')
    '{"version":1,"todos":[]}' | Set-Content -Encoding utf8 (Join-Path $fixture 'local-evidence/state.json')
    foreach ($name in @('LightDo_1.0.0_x64-setup.exe','LightDo_1.0.0_licenses.zip','SHA256SUMS.txt','LightDo_0.1.0_x64-setup.exe','LightDo_0.1.0_licenses.zip')) {
        'synthetic asset, never executed' | Set-Content -Encoding utf8 (Join-Path $fixture ('dist/' + $name))
    }
    Fixture-Git init --quiet -b main
    Fixture-Git config core.autocrlf false
    Fixture-Git add Cargo.toml .gitignore scripts
    Fixture-Git -c user.name=LightDo-Acceptance -c user.email=acceptance@example.invalid commit --quiet -m fixture
    $manifest = Join-Path $fixture 'Cargo.toml'
    $originalManifest = [IO.File]::ReadAllBytes($manifest)
    '# intentional dirty fixture' | Add-Content -Encoding utf8 $manifest
    Expect-Refusal '工作区有未提交修改'
    [IO.File]::WriteAllBytes($manifest, $originalManifest)
    Fixture-Git add -f target/sentinel.txt
    Expect-Refusal '含已跟踪文件'
    Fixture-Git rm --cached --quiet target/sentinel.txt
    $keep = Join-Path $fixture 'dist/LightDo_1.0.0_x64-setup.exe'
    $keepHash = (Get-FileHash -LiteralPath $keep).Hash
    & (Join-Path $fixture 'scripts/clean-delivery.ps1') -Execute -IncludeTestData | Out-Null
    # No obsolete assets or intermediate directories remain: repeating the
    # command must be harmless and must not turn an empty member list into null.
    & (Join-Path $fixture 'scripts/clean-delivery.ps1') -Execute -IncludeTestData | Out-Null
    foreach ($relative in @('target','.tools','.reference','local-evidence','dist/stage','dist/LightDo_0.1.0_x64-setup.exe','dist/LightDo_0.1.0_licenses.zip')) {
        if (Test-Path -LiteralPath (Join-Path $fixture $relative)) { throw '隔离测试中间文件未清理。' }
    }
    if ((Get-FileHash -LiteralPath $keep).Hash -ne $keepHash -or
        !(Test-Path -LiteralPath (Join-Path $fixture 'scripts/common.ps1')) -or
        !(Test-Path -LiteralPath (Join-Path $fixture '.git')) -or
        !(Test-Path -LiteralPath (Join-Path $fixture 'dist/LightDo_1.0.0_licenses.zip')) -or
        !(Test-Path -LiteralPath (Join-Path $fixture 'dist/SHA256SUMS.txt'))) { throw '隔离测试误删了应保留内容。' }
    [ordered]@{dirtyTreeRefused=$true;trackedTargetRefused=$true;currentAssetsRetained=$true;obsoleteAssetsRemoved=$true;repeatedCleanupSafe=$true;sourceAndGitRetained=$true;cleanupScriptSha256=(Get-FileHash -LiteralPath $cleanupSource).Hash.ToLowerInvariant()} |
        ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $fixtureParent 'test-clean-delivery.json')
    Write-Output 'Cleanup guards passed in an isolated synthetic repository; actual project was not cleaned.'
} finally { Set-Location -LiteralPath $originalLocation.Path }
