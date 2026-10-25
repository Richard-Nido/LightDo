. "$PSScriptRoot/common.ps1"
$paths = & git ls-files -- .cargo Cargo.toml Cargo.lock rust-toolchain.toml build.rs src assets installer scripts .github
if ($LASTEXITCODE -ne 0) { throw '无法读取受版本控制的源码清单' }
$parts = foreach ($path in $paths | Sort-Object -CaseSensitive) {
    $hash = & git rev-parse "HEAD:$path"
    if ($LASTEXITCODE -ne 0) { throw "无法读取已提交源码对象：$path" }
    "$path=$hash"
}
$bytes=[Text.Encoding]::UTF8.GetBytes(($parts -join "`n")+"`n")
[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
