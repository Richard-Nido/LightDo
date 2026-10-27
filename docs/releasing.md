# 开发与发布

正式标签要求本机玻璃、窗口模式、中文输入和安装验收通过，并绑定已提交源码指纹；不能以编译成功代替。本机核心结果已记录，公开Release状态及最终下载结果以发行页和verification.md后续记录为准。

本机使用 Rust 1.99.1 x86_64-pc-windows-msvc、Visual Studio 2022 Build Tools 17.14.42（C++ 桌面工作负载）及 Windows SDK 10.0.26100。构建脚本使用 PowerShell 7；运行应用本身不需要 PowerShell 或开发工具。

```powershell
.\scripts\dev.ps1 -Isolated
.\scripts\check.ps1
.\scripts\build.ps1
```

隔离开发数据位于 `local-evidence/dev-data`，不会写入正式用户目录。

当前用户的实际重启保留可用`scripts/test-restart.ps1 -Executable "$env:LOCALAPPDATA/Programs/LightDo/lightdo.exe" -Cycles 3`检查。它需要现有非空清单，只正常退出、从不同工作目录启动并核对数量与文件SHA，不创建或删除任务；有其他实例、退出失败或数量不符时停止保留现场。必须在没有正在编辑任务时运行；它不替代实际界面观察。

运行 `cargo run --release --example glass_background` 打开本项目的动态无隐私测试窗口。它是测试工具，不随应用安装，不作为应用内置背景，也不会在正式运行时启动。

发布者为 Yuan Haoran (Richard-Nido)，无代码签名证书。发布时应如实说明安装包未签名以及 Windows 可能显示发布者验证或 SmartScreen 提示；不得要求用户关闭安全防护。

安装器使用 [NSIS 官方 3.12 ZIP](https://sourceforge.net/projects/nsis/files/NSIS%203/3.12/nsis-3.12.zip/download)，锁定 SHA-256 `56581f90db321581c5381193d796fffcf2d24b2f8fed2160a6c6a3baa67f2c4f`。`scripts/get-nsis.ps1` 从官方镜像取得并校验，校验失败即停止。

```powershell
.\scripts\get-nsis.ps1
.\scripts\package.ps1
.\scripts\test-installer.ps1
```

安装器默认当前用户安装，不需要管理员权限，默认安装到 `%LOCALAPPDATA%\Programs\LightDo`。支持开始菜单和可选桌面快捷方式；应用运行中返回 1618，并要求先正常退出，不强杀。升级保留 `%APPDATA%\LightDo`；卸载默认保留数据，只有主动勾选才删除。运行时使用静态 CRT，不要求安装额外 VC 运行库。

静默安装冒烟脚本只在本应用尚无安装、数据、自启或菜单目录时运行，避免覆盖真实用户内容。它测试安装、同版本覆盖、卸载登记和默认保留数据标记，并清理自己的测试标记；不等于 GUI 或跨版本升级验收。

版本更新和发布使用同一个脚本：

1. 提交代码后，运行 `scripts/release.ps1 -Prepare -Version 1.0.0`，更新源码/资源版本并检查、打包和提交准备版本。
2. 对该 EXE 完成本机实际验收，更新 `docs/verification.md` 及 `docs/release-acceptance.json`；只有实际通过的项目才能置为 true。提交最终验收记录后运行 `scripts/finalize-acceptance.ps1`，它会对不含自身的已提交源码清单生成指纹并提交验收门槛。
3. 运行 `scripts/release.ps1 -Publish`。脚本阻止脏工作区、版本不一致、验收指纹不匹配、未通过项目和重复标签。
4. 标签触发 Windows Actions：核验门槛、检查、测试、构建、打包、生成许可附件与 SHA-256；发布任务先创建草稿并上传全部资产，重新下载逐一核对SHA-256后，才改为公开正式Release。只有发布任务有 contents:write。
5. 下载 Release EXE 对照 SHA-256 并复核安装运行，把实际公开状态、下载哈希与安装结果追加到verification.md。

已经公开的版本不改写标签；修复源码后提高补丁版本再发布。发布前未通过的构建可直接修复主分支，不能伪造已通过状态。未签名状态需在每次 Release 说明中保留。

上传或草稿下载校验失败会停止公开步骤。确认失败版本仍为草稿、没有公开版本后，可删除该失败草稿（保留原标签）再重跑发布任务；脚本不覆盖已有Release。`publish-release.ps1 -ValidateOnly`仅检查本地六项资产、版本与校验值，不联网或发布。

构建和打包均遵守 CARGO_TARGET_DIR；调试运行的 EXE 被占用时，可在独立构建目录准备候选，不替换正在运行的程序。打包会核对 EXE 资源版本与源码版本，防止误用旧二进制。

完成正式 Release 下载、校验和安装复核后，执行工作目录清理：

```powershell
.\scripts\clean-delivery.ps1
.\scripts\clean-delivery.ps1 -Execute -IncludeTestData
```

第一条只列出计划，不删除文件。第二条用于本项目已确认仅含合成任务的隔离验收数据；如果其中有实际需要的任务，先保留到本应用正常数据目录，不能把实际任务当中间文件删除。脚本会先检查 Git 根目录、已跟踪文件、目录链接、未提交修改和正在运行的工作区 EXE，全部满足后才删除 target、.tools、.reference、local-evidence 和 dist/stage。Git 历史、源码、文档、许可、dist 安装包及校验文件保留，项目外目录不在清理范围。

最终清理现在只保留Cargo当前版本的安装包、许可ZIP及校验文件，也会识别旧0.1.0开发资产；不会把旧开发EXE作为必要交付物长期留下。check.ps1先检查所有PowerShell脚本语法，并在独立合成仓库运行清理安全测试：拒绝脏工作区和已跟踪目标，实际清除合成中间目录/旧资产，同时保留当前资产、源码和Git。本项目自身仍须等正式Release下载、校验及安装验收完成后才执行清理。
