# 交付收尾

正式v1.0.0已公开，源码标签提交c2c476d9f1d2bfdca00f71faf33af19a0705b1b6，标签工作流40447543051的build/publish均成功。Release draft=false/prerelease=false，6项资产已重新下载并逐项匹配GitHubAPI SHA-256及SHA256SUMS。

最终公开EXE安装包371488字节，SHA831e093c8d9f57dc26385bacdba25ea8a1b751035af9540abb62674be127a576。已使用该下载包安装D:\LightDo，应用SHA9a83383752ac6c4019c3e2bf9c4ab9b48601cf6ea36fd9fc69367de023aca8a2，1.0.0/NotSigned，3次冷启动保留任务/设置，用户明确“正式版正常！”确认玻璃、中文输入和数值8。当前6条任务，原5个任务ID仍在，新增正文仅留本机。

当前源码默认Right、停靠范围16；用户自定义8已保留。安装准备曾恢复Right，用户后续手动停靠Left，保留其当前选择，不强制覆盖。最终用户关闭面板为TrayHidden，测量明确属于托盘隐藏，不能冒充日常边缘自动隐藏。各构建/场景记录见performance.md，代表测量JSON已移至docs/performance-records。

所有临时AppData保护目录均已还原，原有及新增任务均在正式%APPDATA%/LightDo。不要从local-evidence旧副本覆盖当前数据。用户要求避免Computer Use，已经采用CLI/文件/诊断与人工验收，未再恢复自动桌面控制。

唯一剩余：实际工作目录清理、清理后校验和最终完成审计。请先提交当前正式文档/JSON，再运行scripts/clean-delivery.ps1 -Execute -IncludeTestData；删除target/.tools/.reference/local-evidence/dist/stage及旧0.1资产，保留源码/Git、正式文档/图片和dist当前公开EXE/许可ZIP/SHA。不得删除D盘安装、AppData或其他项目。

GitHub CLI已从项目临时目录迁到%LOCALAPPDATA%/Programs/GitHub CLI/gh.exe，原二进制校验一致、原Richard-Nido登录保留、原本引用本项目缓存的两个Git helper路径已更新，用户PATH已加入。清理后请使用稳定CLI路径；不要使用即将删除的.tools/gh。

清理后更新release-validation.json的cleanup.executed、verification和清单，提交推送；核对git status干净、所列中间目录及旧资产不存在、当前dist SHA正确、D盘应用及AppData任务数/哈希未受影响。源标签保持不改写，main后续只有交付记录推进。然后给用户仓库/Release/EXE链接、本机路径/版本/源码提交/安装包SHA、三模式图和真实测试范围，确实完成后update_goal complete。
