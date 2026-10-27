# 当前接续点

完整目标仍为LightDo完整产品、本机真实验收、Win11 x64 EXE、独立公开仓库、正式标签自动公开Release、最终下载校验安装和目录清理。建仓/推送/发布已授权。当前没有正式标签或Release，不能宣称完成。

## 最新现场（2026-10-27，核心验收完成后）

- 最新用户新增要求：悬浮拖回边缘距离更小且可设置。已实现默认48→16DIP，窗口页“拖动停靠范围”可4–96，桌面模式禁用；自定义拖动预览/松手及系统移动结束共用geometry::snap_side，统一缩放和近侧优先。
- 新安装版D:\LightDo SHA8d47bccfe1f868e1308fe38ea66ade985f3594a5b1a4e463e4223a25b3b4345d，PID37184，原5条任务/原有偏好及安装器覆盖前后旧文件字节保持。旧JSON读默认16，保存前磁盘新字段可null；不能当成0。
- 已向用户异步请其在设置窗口页确认默认16，改8应用，悬浮拖近边缘检查更近才预览/松手停靠，再确认8保留。待回复后读JSON、正常退出重启验证新值及5条任务，再恢复windowModes=true。正式发布还没创建tag。
- 新37测试中28通过、9忽略；已构建打包371659字节、安装哈希与stage相同。新增旧数据默认/自定义往返/安全范围、各DPI左右边界及负坐标/近侧回归。
- 用户最后桌面截图已收到并裁存docs/images/desktop-current.png；当时同PID15340/5条/Desktop，1面板1控制，约3轮模式手测“一切正常”，WS111.21→113.02、私有89.92→91.07MiB、句柄502→506，仅短时样本。此图是7b1b879a构建，不冒充新增距离后的资源/GUI实测。
- 下方关于最后截图待提供的历史步骤已完成；下一步先完成新增距离实测，然后提交/finalize/推正式tag。新代码验收未完成前保持status=pending、windowModes=false，源码指纹仍null。

- 用户要求尽量避免Computer Use；物理Esc此前已停止控制，之后全部采用CLI/文件/进程诊断和用户人工验收，不擅自恢复CUA。
- 唯一当前应用D:\LightDo\lightdo.exe，版本1.0.0、SHA7b1b879a8bff6f2a89b8ebb048f667311b43d2a5807d0bc50c22643458c5126f，PID15340；正式数据%APPDATA%/LightDo/state.json，5条未完成，Edge/Right/Light48/350×547/CtrlAltL/自启关闭。
- 所有原数据保护目录都已完整还原，AppData旁没有未恢复LightDo.acceptance目录。不要再从local-evidence旧副本覆盖正式文件。只剩隔离测试数据及证据，最终清理会删除。
- 最新用户反馈：任务可见/菜单保持正常；边缘快速掠过、停留呼出、离开后快速返回约3轮“边缘手势正常”。
- 主动删除卸载第一轮用户忘记勾选，未记PASS；第二轮明确“已勾选并卸载”，实际合成数据目录、应用和卸载登记消失，保护原数据SHA不变。之后自动重装D盘，逐字节恢复原数据，3次冷启动均5条，内容和SHA不变。
- 保存失败GUI：最初180秒共享锁超时，用户正常添加，判无效注入。改隔离state.json只读后用户确认失败提示；实际文件仍2条/SHA不变，内存4条、临时文件0，正式5条不变。解除只读、正常退出后4条全保存、原2条ID保持。
- 损坏读取GUI：隔离无效JSON，diagnostic readable=false，用户确认提示/输入禁写/原文件保留说明正常；退出前后坏字节SHA不变。已退出隔离应用、恢复正式D盘5条，文件SHA与测试前一致，无遗留锁或只读属性。
- 先前用户WinD往返、睡眠恢复/组件输入/图标正常；Explorer真实重启14816→18580，同一轻待进程38136回Floating/非置顶/可见、5条、背景就绪，是可用悬浮恢复而非自动继续Desktop。
- 34测试中25通过、9真实桌面实验忽略。main上0b4e781的CI40441239032已成功，之后只有验收/README等文档改动，没有应用二进制改变。
- docs/release-acceptance.json四核心标记已据实置true，但status仍pending、sourceFingerprint=null：待最终视觉证据、提交后finalize，不得直接提前推tag。

## 下一步（当前正在请求最后模式图）

1. 已预热正式Edge玻璃，mode-cycles-before.json：PID15340、5任务、WS111.21MiB/私有89.92MiB/句柄502/线程13。请用户菜单Edge→Floating→Desktop往返约3轮，任务一致且一个面板，最后停Desktop、回桌面并提供仅含轻待+少量桌面的截图。已有Edge和Floating实际图片，缺Desktop交付图；遵循用户偏好人工提供，不用CUA。截图只在普通final消息请求，request_user_input_async不支持要求上传图片。
2. 收到图后先核验当前诊断/同PID/模式/任务，再查看用户图片、裁去无关或隐私区域，保存docs/images内并记录来源与模式范围。仅短时3轮不称长期无泄漏。复核重复窗口/资源计数，不伪造长期结果。
3. 更新README/CHANGELOG/Release说明/verification与清单为本机实际通过，明确单物理显示器、仅Win11 26H2、未签名、注销启动等未测限制。验收当前源码提交后finalize-acceptance生成指纹并提交。
4. 运行release -Publish，跟踪正式v1.0.0标签Actions直到公开draft=false/prerelease=false、6项资产齐全。不要覆盖已公开标签。下载全部资产核SHA，最终公开安装器安装D:\LightDo（正常退出，保留任务），运行诊断及必要用户最终UI确认。
5. 最终Release下载安装验收后再实际清理target/.tools/.reference/local-evidence/dist/stage和旧0.1资产，保留源码/Git/正式文档/当前版本交付资产；AppData和D盘安装不在清理范围。清理专项合成测试已通过，但真实项目未执行过。最后给源码/Release/EXE链接、版本/提交/SHA、本机EXE路径、3模式真实图和真实测试范围。

## 关键工具约束

- Windows GUI EXE --diagnose直接管道ConvertFromJson，不先赋值再解析，以免stdout管道提前关闭。正常退出用StartProcess -Wait -PassThru读取ExitCode，等进程结束；禁止强杀LightDo。
- test-restart.ps1完整比较解析JSON的全部任务与设置，另报告SHA；格式变化不当数据丢失。本机最新多次内容和字节都保持。
- Rust1.99.1/VS17.14.42/SDK26100/NSIS3.12，gh .tools/gh/bin/gh.exe。CARGO_TARGET_DIR=target/candidate；root app旧，root玻璃fixture最新双缓冲--fullscreen/--static。
- 发布脚本先草稿上传/下载逐一SHA核对再公开，失败保留草稿；最小权限只有publish job contents:write。正式tag指向验收源码，不能把本地旧EXE当最终下载版验收。
- 原始本机JSON都在local-evidence，不含任务正文的诊断可供核对；最终删除中间数据前先确认正式AppData5条及用户后续新增任务已保留。
