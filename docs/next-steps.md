# 当前接续点

完整目标：LightDo完整产品、本机真实验收、Win11 x64 EXE、独立公开仓库、正式标签自动公开Release、最终下载校验安装和目录清理。建仓/推送/正式发布已授权。尚无正式标签或Release，验收JSON仍pending。

## 最新现场（2026-10-27，用户手动装至D盘后）

- 用户明确要求后续尽量避免Computer Use，改为命令行/文件/真实进程诊断优先；需要UI时给用户简短手工操作。此前物理Esc已停止Computer Use，不擅自恢复GUI自动控制。
- 用户人工安装至D:\LightDo，EXE版本1.0.0，SHA7b1b879a8bff6f2a89b8ebb048f667311b43d2a5807d0bc50c22643458c5126f，与dist/stage一致。卸载登记指向D:\LightDo\uninstall.exe，开始菜单入口存在。
- 原有5条未完成任务已从保护目录完整恢复，任务ID/正文/完成状态保持不变。先按保护SHA逐字节恢复，再只将模式/侧恢复Edge/Right。不要再从旧副本覆盖当前正式文件。
- 当前正式数据%APPDATA%/LightDo/state.json，5条未完成，Edge/Right/Light48/350×547/CtrlAltL/自启关闭。保护目录LightDo.acceptance-backup-5ab...已恢复到原位置，不再存在；local-evidence/installer-gui-protection.json的restored=true。
- D盘实际安装版3次完整退出/冷启动PID14436/23260/9420均readable=true、tasks=5、unfinished=5，任务设置内容与本轮SHA不变，约311/243/235ms至数据/输入控件创建（含200ms轮询与诊断子进程）。当前应为PID9420，执行前重新检查。
- 已异步请用户按CtrlAltL确认5条任务实际可见，并打开菜单后移开鼠标确认菜单保持。正在等待结果，不要因为命令行可读就代替用户视觉结果。

## 最近真实通过

- 用户睡眠恢复、组件输入和桌面图标正常；WinD往返早前新版已手测通过。
- 用户手动重启Explorer后反馈自动恢复成功；实际Explorer14816→18580，同一LightDo PID38136恢复Floating/非置顶/可见，5条任务可读、真实背景就绪。是明确恢复悬浮降级，不假称自动保持Desktop。
- 默认卸载实际数据页可见，删除数据默认未勾。现有数据先移动保护并校验，只用一条合成标记。用户授权后向导自行结束（不是工具最终点击）；实际应用/卸载登记/开始菜单/测试自启全部清理，合成数据SHA及保护原始SHA不变。
- 随后安装欢迎页被识别，但用户物理Esc停止自动控制，未记工具GUI安装通过。现在安装完成来自用户手动说明+D盘真实文件/登记/运行验证。
- 多任务32条隔离清单滚到末尾、Tab回可见首项、Space完成/撤销原位、中文Emoji全选复制连续粘贴3段、Enter新增、空白不新增通过。用户授权单条Delete后33→32、其他ID不变、正式5条SHA不变；授权已用完，不扩大。
- 最新34测试中25通过、9真实桌面实验忽略；PowerShell语法/隔离清理专项/fmt/check通过。当前EXE含数据目录补强、短锁重试、明确读失败空态、设置文件夹入口、列表键盘保持、内部鼠标Esc收回修正。
- 最新安装版右边缘隐藏38.95秒、37样本均隐藏，CPU0.0120%、GPU0%、工作集98.76/结束95.83MiB、私有72.02、共享GPU87.57、句柄531，哈希范围见performance.md。
- main已推e221ede，CI40438071297成功；42c2273 CI40437515095也成功。当前只有后续文档、test-restart.ps1语义比较修正待提交，无应用二进制变动。

## 重要未完成

1. 等用户五条任务可见/菜单保持反馈；必须人工交互的剩余边缘快速掠过/离开返回/反向等可短清单一次验证，不用Computer Use反复操作。
2. 卸载“主动删除数据”尚未实际验证。用户刚装好D盘，不能直接删除当前5条。需要先正常退出、完整保护正式目录，再准备仅一条合成任务，请用户在D盘卸载向导勾选主动删除并卸载，之后核验目录与登记清理；再安装并恢复原始数据。按用户偏好用人工操作，避免CUA。此前只授权默认保留卸载，不扩展。
3. 写失败/损坏保护已有单元测试与禁写逻辑，真实GUI提示仍待；可用隔离合成状态+受控文件锁，请用户确认提示，不涉及原5条。
4. 安装器完整本机验收结束，三个P0/本机关键门槛实际通过后才更新release-acceptance、finalize、release -Publish。标签Actions构建检查/草稿上传/下载逐文件校验后公开。随后下载最终公开资产SHA核对、从最终包安装运行，再实际清理target/.tools/.reference/local-evidence/dist/stage与旧版资产。
5. 完善正式README/Release说明/验证矩阵，最终给源码/Release/EXE链接+SHA/真实测试范围。单物理显示器与仅本机Win11 26H2如实列限制，不伪造外接屏断开或多版本保证。

## 保护与工具

- D盘安装版为唯一当前正常应用，正式AppData必须保留。local-evidence/ux-fixture-current.json是已退出隔离32条清单，不能混入正式数据；最终清理会删除中间数据。
- test-restart.ps1现在完整比较解析后的任务和设置内容，另报告字节SHA；PowerShell/Rust空白规范化不能当数据丢失，内容变化仍立即失败保留现场。3次最新D盘测试内容和字节均保持。
- Windows GUI EXE --diagnose直接管道ConvertFromJson，不先赋值再管道以免关闭stdout。正常退出StartProcess -Wait -PassThru读取ExitCode并等进程结束，禁止强杀LightDo。
- 如后续获准恢复Sky，只用mcp__node_repl__js的@oai/sky、globalThis读写、从list_windows唯一选择并用state.window刷新。禁止Windows键、Codex/认证/安全UI，物理Esc或锁屏立刻停止；当前优先完全不用。
- Rust1.99.1/VS17.14.42/SDK26100/NSIS3.12；gh .tools/gh/bin/gh.exe；CARGO_TARGET_DIR=target/candidate。root app旧，root玻璃fixture是最新双缓冲且支持--fullscreen/--static。
- 清理专项在独立合成仓库通过，实际项目尚未清理。最终清理须先完成公开Release下载安装复核，并保证正式AppData已恢复、不含未迁出的真实任务。脚本仅清工作区，不删D盘安装或AppData。
