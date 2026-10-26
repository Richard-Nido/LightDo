. "$PSScriptRoot/common.ps1"
if ((& git status --porcelain).Length -gt 0) { throw '工作区必须干净；先提交源码和最终验收记录。' }
$metadata=& (Find-Cargo) metadata --locked --format-version 1 --no-deps
if ($LASTEXITCODE -ne 0) { throw '无法读取源码版本' }
$version=(($metadata|ConvertFrom-Json).packages|Where-Object name -eq 'lightdo').version
$acceptance=Get-Content -Raw 'docs/release-acceptance.json'|ConvertFrom-Json
if ($acceptance.version -ne $version) { throw '验收记录版本与 Cargo.toml 不一致' }
foreach($field in @('glass','windowModes','chineseIme','installer')){if($acceptance.$field -ne $true){throw "验收项目尚未全部设为 true：$field"}}
$fingerprint=& "$PSScriptRoot/source-fingerprint.ps1"
$acceptance.status='passed';$acceptance.sourceFingerprint=$fingerprint
$acceptance | ConvertTo-Json | Set-Content -Encoding utf8 'docs/release-acceptance.json'
Invoke-Checked git add docs/release-acceptance.json
Invoke-Checked git commit -m "chore: record LightDo $version local acceptance fingerprint"
Write-Output $fingerprint
