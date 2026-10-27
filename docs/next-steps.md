# 发布接续点

本机核心验收已经完成，新增停靠范围亦由用户确认正常并明确保留8。现在只推进正式标签Release、最终公开下载安装和实际目录清理，不再重复产品询问或已通过的桌面验收。

- 当前D:\LightDo\lightdo.exe，1.0.0候选SHA8d47bccfe1f868e1308fe38ea66ade985f3594a5b1a4e463e4223a25b3b4345d；最新进程24016，正式AppData5条未完成，snap_distance=8。用户拖动测试后为Edge/Left，最终安装验收可恢复原要求的Right，同时保持8及全部任务。
- 默认16、范围4–96；设置显示、改8、预览和松手、重新打开8保存均获用户确认，3次冷启动仍8和5条、内容与SHA不变。37项中28通过、9真实桌面实验忽略。f11eec8 CI40446715246全部成功。
- 真实Desktop图已收到并存docs/images/desktop-current.png（用户提供，仅裁无关桌面），与当时同PID15340/Desktop/5任务诊断对应；Edge/Float图片已存在。图的7b1b879a构建范围已说明，不冒充新停靠参数的直接资源实测。
- 真实数据读/写失败、原文件不被覆盖、解除只读后保存成功、默认保留及主动删除卸载、恢复原5条、用户D盘安装、Explorer同进程回Floating恢复、WinD/睡眠/图标/IME各DPI等结果均见verification.md；不补造单物理屏无法实测的外接断开等条件。
- 用户要求尽量不用Computer Use，且之前物理Esc停止控制。保持CLI/文件/诊断优先，需要最终公开下载版显示/中文输入时给用户一次简短人工确认，不自动恢复CUA。
- 正式AppData为唯一用户数据权威，所有临时保护目录已还原，无遗留锁/只读。不得用local-evidence旧副本覆盖当前任务；清理只在工作区，绝不删D盘安装或AppData。

## 接下来顺序

1. 提交本次最终验收文档和all-true的pending门槛；finalize-acceptance.ps1生成指纹并提交；用release.ps1 -Publish推v1.0.0，已获用户授权，不再征求发布确认。此前无正式tag/Release，不覆盖已公开版本。
2. 跟踪标签Actions，核实draft=false/prerelease=false、安装包/许可ZIP/SHA/项目LICENSE/第三方声明/依赖清单六项资产完整可下载。失败按真实退出码处理；不要把本地预检当真实公开发布。
3. 下载最终公开资产，核对SHA，正常退出D盘当前进程，再把最终下载安装器覆盖到D:\LightDo，保留全部任务和snap_distance=8，恢复Right偏好。运行/重启诊断并请用户一次实际显示与中文新增确认。必要时做最终公开构建边缘隐藏采样，所有结果明确哈希范围。
4. 更新公开Release下载验收记录、README直达链接和清单。正式tag保留当时验收源码，不改写标签；后续交付文档提交可在main。
5. 保存最终安装包/许可ZIP/SHA到dist，移出最终需要的证据至正式docs后，实际执行clean-delivery -Execute -IncludeTestData。先干净Git、验证没有工作区运行EXE；确认所有保护数据已还原，保留正式AppData当前任务。删除target/.tools/.reference/local-evidence/dist/stage及旧0.1资产，核实目录整洁和正式文件/安装/数据未受影响。
6. 最终回答源码/Release/EXE链接、版本/tag提交SHA/安装包SHA、本机D盘EXE路径、三模式图/测试结果、默认16当前8与维护发布步骤。只有全部完成才update_goal complete。

工具：Rust1.99.1/VS17.14.42/SDK26100/NSIS3.12；gh .tools/gh/bin/gh.exe，CARGO_TARGET_DIR=target/candidate。WindowsGUI诊断直接管道ConvertFromJson；退出用StartProcess -Wait读ExitCode，禁止强杀。调用纯PS验证脚本要按真实异常/独立pwsh退出码判断，不拿旧LASTEXITCODE误报。清理后gh缓存会删除，因此完成下载/公开API复核再清理。
