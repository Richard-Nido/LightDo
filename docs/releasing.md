# 开发与发布

当前尚未发布。正式标签被本机玻璃、窗口模式、中文输入和安装验收门槛阻止；不能以编译成功代替。

本机使用 Rust 1.99.1 x86_64-pc-windows-msvc、Visual Studio 2022 Build Tools 17.14.42（C++ 桌面工作负载）及 Windows SDK 10.0.26100。构建脚本使用 PowerShell 7；运行应用本身不需要 PowerShell 或开发工具。

```powershell
.\scripts\dev.ps1 -Isolated
.\scripts\check.ps1
.\scripts\build.ps1
```

隔离开发数据位于 `local-evidence/dev-data`，不会写入正式用户目录。

运行 `cargo run --release --example glass_background` 打开本项目的动态无隐私测试窗口。它是测试工具，不随应用安装，不作为应用内置背景，也不会在正式运行时启动。

发布者为 Yuan Haoran (Richard-Nido)，无代码签名证书。发布时应如实说明安装包未签名以及 Windows 可能显示发布者验证或 SmartScreen 提示；不得要求用户关闭安全防护。

安装器使用 [NSIS 官方 3.12 ZIP](https://sourceforge.net/projects/nsis/files/NSIS%203/3.12/nsis-3.12.zip/download)，锁定 SHA-256 `56581f90db321581c5381193d796fffcf2d24b2f8fed2160a6c6a3baa67f2c4f`。`scripts/get-nsis.ps1` 从官方镜像取得并校验，校验失败即停止。

```powershell
.\scripts\get-nsis.ps1
.\scripts\package.ps1
```

安装器默认当前用户安装，不需要管理员权限，默认安装到 `%LOCALAPPDATA%\Programs\LightDo`。支持开始菜单和可选桌面快捷方式；应用运行中返回 1618，并要求先正常退出，不强杀。升级保留 `%APPDATA%\LightDo`；卸载默认保留数据，只有主动勾选才删除。运行时使用静态 CRT，不要求安装额外 VC 运行库。

版本更新和发布使用同一个脚本：

1. 提交代码后，运行 `scripts/release.ps1 -Prepare -Version 1.0.0`，更新源码/资源版本并检查、打包和提交准备版本。
2. 对该 EXE 完成本机实际验收，更新 `docs/verification.md` 及 `docs/release-acceptance.json`；只有实际通过的项目才能置为 true。运行 `scripts/source-fingerprint.ps1` 记录已提交源码指纹，提交验收记录。
3. 运行 `scripts/release.ps1 -Publish`。脚本阻止脏工作区、版本不一致、验收指纹不匹配、未通过项目和重复标签。
4. 标签触发 Windows Actions：核验门槛、检查、测试、构建、打包、生成许可附件与 SHA-256，全部成功后才创建公开 Release。只有发布任务有 contents:write。
5. 下载 Release EXE 对照 SHA-256 并复核安装运行。当前尚未走到正式发布这一步。

已经公开的版本不改写标签；修复源码后提高补丁版本再发布。发布前未通过的构建可直接修复主分支，不能伪造已通过状态。未签名状态需在每次 Release 说明中保留。
