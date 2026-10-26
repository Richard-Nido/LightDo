param([string]$Version, [switch]$Prepare, [switch]$Publish, [switch]$CheckOnly)
. "$PSScriptRoot/common.ps1"
function Get-Version {
    $text=& (Find-Cargo) metadata --locked --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) { throw '无法读取源码版本' }
    (($text | ConvertFrom-Json).packages | Where-Object name -eq 'lightdo').version
}
if ((& git status --porcelain).Length -gt 0) { throw '工作区不干净。请先检查并提交全部源码和验收记录。' }
if ($Prepare) {
    if ($Version -notmatch '^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$') { throw '必须指定三段 SemVer -Version' }
    & git show-ref --verify --quiet "refs/tags/v$Version"
    if ($LASTEXITCODE -eq 0) { throw '该版本标签已存在，禁止改写。' }
    $manifest=Get-Content -Raw -LiteralPath 'Cargo.toml'
    $manifest=[regex]::Replace($manifest,'(?m)^version = "\d+\.\d+\.\d+"\r?$',"version = `"$Version`"",1)
    [IO.File]::WriteAllText((Join-Path $script:ProjectRoot 'Cargo.toml'),$manifest,[Text.UTF8Encoding]::new($false))
    Invoke-Checked (Find-Cargo) update -p lightdo
    $acceptance=Get-Content -Raw 'docs/release-acceptance.json' | ConvertFrom-Json
    $acceptance.status='pending';$acceptance.version=$Version;$acceptance.sourceFingerprint=$null
    foreach ($field in @('glass','windowModes','chineseIme','installer')) { $acceptance.$field=$false }
    $acceptance | ConvertTo-Json | Set-Content -Encoding utf8 'docs/release-acceptance.json'
    & "$PSScriptRoot/check.ps1"
    if ($LASTEXITCODE -ne 0) { throw '检查失败，未继续打包或提交准备版本。' }
    & "$PSScriptRoot/package.ps1"
    if ($LASTEXITCODE -ne 0) { throw '打包失败，未提交准备版本。' }
    Invoke-Checked git add Cargo.toml Cargo.lock docs/release-acceptance.json
    Invoke-Checked git commit -m "chore: prepare LightDo $Version for local acceptance"
    Write-Output '准备完成。请测试这份安装包并更新验收记录；验收完成后提交，再运行 -Publish。'
    return
}
$current=Get-Version
if ($Version -and $Version -ne $current) { throw '指定版本与源码版本不一致，请先使用 -Prepare' }
$tag="v$current"
if ($env:GITHUB_REF_TYPE -eq 'tag' -and $env:GITHUB_REF_NAME -ne $tag) { throw '标签与源码版本不一致' }
$acceptance=Get-Content -Raw 'docs/release-acceptance.json' | ConvertFrom-Json
$fingerprint=& "$PSScriptRoot/source-fingerprint.ps1"
if ($acceptance.status -ne 'passed' -or $acceptance.version -ne $current -or $acceptance.sourceFingerprint -ne $fingerprint) { throw '本机验收未通过，或验收源码指纹/版本不一致。禁止正式发布。' }
foreach ($field in @('glass','windowModes','chineseIme','installer')) { if ($acceptance.$field -ne $true) { throw "缺少真实验收：$field" } }
if ($CheckOnly) { Write-Output "Release gate passed: $tag"; return }
if (!$Publish) { throw '请指定 -Prepare、-Publish 或 -CheckOnly' }
Invoke-Checked git fetch origin --tags
& git show-ref --verify --quiet "refs/tags/$tag"
if ($LASTEXITCODE -eq 0) { throw '该版本标签已存在，禁止改写。' }
& "$PSScriptRoot/check.ps1"
if ($LASTEXITCODE -ne 0) { throw '检查失败，未推送或创建正式标签。' }
Invoke-Checked git push origin main
Invoke-Checked git tag -a $tag -m "轻待 LightDo $current"
Invoke-Checked git push origin $tag
