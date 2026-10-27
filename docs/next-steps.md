# 当前接续点

完整目标：完成LightDo、本机真实验收、Windows安装包、独立公开仓库、正式标签自动公开Release、下载校验安装、最终清理。已获建仓/推送/发布授权，不重复询问产品需求。尚无正式标签或Release，验收JSON仍pending，不能宣称完成。

## 最新现场（2026-10-27 09:20后）

- 当前安装版1.0.0 SHA-256 7b1b879a8bff6f2a89b8ebb048f667311b43d2a5807d0bc50c22643458c5126f，路径%LOCALAPPDATA%/Programs/LightDo/lightdo.exe，PID38136，最新3次冷启动的第三个进程。
- 当前已通过菜单实际切到Desktop（为Explorer重启测试），toolWindow=true/topmost=false、5条任务；Right偏好/Light48/350×547/CtrlAltL/自启关闭。全屏测试背景已关闭，设置已关闭。
- 系统locked=false。Taskmgr已打开，工具操作搜索框/快捷键不可靠，没有执行资源管理器重启。已经异步请求用户手动在任务管理器Windows资源管理器上点“重新启动”，等待结果。用户操作期间暂不操作GUI；先诊断再刷新Sky。
- 用户已确认“睡眠恢复，组件输入，桌面图标都正常”。仅记用户手测，不声称工具证明同一进程跨睡眠。
- 用户报告重启任务不可见：原AppData仍5条未完成任务，旧版重新启动又显示。原先触发原因尚未确诊；已做读取补强，不宣称精确根因。
- 正式AppData/LightDo/state.json仍5条任务，最新基线SHA4B6A051F77A8A141CE0058F4A43AD99292F0CE41B348E8E88A6E85670E2CA945。正在Desktop切换会改变设置SHA，后续基线应重新读当前文件，不能用旧文件覆盖。
- 当前安装器370817字节；dist/stage与已安装EXE一致，未签名。还没有最终公开下载资产。

## 最近完成

- Known Folder固定默认本机LightDo数据根、500ms有界共享锁重试、读取失败明确空态/禁写；设置通用页“打开LightDo数据文件夹”实际打开Explorer，看到state.json。开发覆盖必须绝对路径。dev.ps1 finally恢复旧覆盖环境，避免隔离路径泄露到后续启动。
- 最新安装版完整退出/冷启动3次PID40356/19508/38136，从TEMP工作目录均读5条未完成，原文件SHA不变。至数据/原生输入控件创建的观测410/252/239ms，含200ms轮询及诊断子进程，不是首帧玻璃显示延迟。
- 隔离32条任务真实滚动至末尾，修正Tab选中屏幕外首项与滚动上限当帧坐标；完成/撤销原位，完整CtrlA高亮、中文Emoji复制与连续粘贴3段、Enter新增、纯空白不新增均实际通过。
- 用户授权只删新增的三段“选区粘贴测试🙂”这一条，实际Delete成功，33→32、目标ID消失、剩余种子ID一致，正式文件SHA未变。该授权已完成，不可扩大到其他GUI删除。
- 列表焦点未阻止自动隐藏导致第一次Delete时窗口先隐藏，数据仍33。修正后鼠标在外面、Tab选中列表，1.3秒后仍Expanded/foreground=true。
- Esc原被内部鼠标反向打开，已修正明确收回优先；实际鼠标在原面板[2218,637]按Esc后650ms隐藏。新增单元回归。
- 最新检查34项中25通过、9真实桌面实验忽略；PowerShell语法/隔离清理专项/fmt/check也通过。
- 最新安装版日常Edge/Right隐藏38.95秒，37样本全部隐藏，CPU0.0120%、GPU0%，工作集平均98.76/结束95.83MiB，私有72.02、共享GPU87.57MiB、句柄531。历史各哈希仍保留范围，不套用。
- 前一33fe9b4d静态网格可见31.34秒CPU1.60%、GPU5.41/10.34%、工作集114.82/私有90.64/共享109.07MiB。输入焦点/桌面系统动态不能称整个系统完全静止。
- 前两批fb85758 CI40428648501、2250a24 CI40429860882均success；最新Esc/列表焦点改动需提交并跟踪CI。

## 接下来

1. 等待Explorer重启手测结果并诊断恢复。桌面宿主失效代码会回Floating并明确提示，符合可恢复路径；不要把普通Floating假称Desktop。恢复后从菜单重新切Desktop并实际操作，再恢复Right。
2. 必要剩余边缘真实快速掠过/返回/反向/菜单防收回、错误保存GUI提示、模式切换资源稳定性；不要对已通过菜单、IME各DPI、WinD、设置、自启登记、快捷键冲突反复询问。
3. 完整安装GUI/卸载默认保留及主动删除/自启清理。实际GUI安装和删除需computer-use动作时确认，先保护正式AppData并准备合成卸载fixture。不要绕过test-installer.ps1的现有用户资料保护。CI静默安装/默认保留已通过，但不冒充本机GUI/主动删除。
4. 满足关键门槛后再finalize-acceptance、release -Publish，观察正式tag Actions，下载SHA核对安装，最后实际清理target/.tools/.reference/local-evidence/dist stage与旧版资产。现在没有Release不能给虚假链接。
5. 一物理显示器无法实测外接断开/主屏变化，如实列限制；本机受测Win11 26H2不外推全部Win11。

## 数据与工具保护

- 正式数据目录%APPDATA%/LightDo，当前全部5条需保留。临时local-evidence/restart-data-protection/state-before-shutdown.json只为验收保护，旧副本不能覆盖后续编辑。
- 隔离UX目录信息local-evidence/ux-fixture-current.json，33→32删除完成、pendingDeleteCompleted=true；隔离实例已正常退出，不得把其任务复制回正式目录。最终清理会删除这些中间数据。
- Sky经mcp__node_repl__js的@oai/sky。只globalThis读写，避免旧顶层fs/sw/ss遮蔽。从list_windows唯一返回对象选择，每次用state.window。原生菜单/设置树常落后一帧，刷新后再操作；截图可能含主窗+第二张设置，不能误判。
- 禁止Windows键、Codex/认证/安全UI；物理Esc或锁定即停止。任务管理器尝试搜索不成功，未改系统进程。已请求用户做一次最小操作。
- Windows GUI EXE诊断必须直接管道ConvertFromJson，先赋值再解析可能不等待进程而关闭stdout。退出/启动需要码时StartProcess -Wait -PassThru读取ExitCode，不沿用旧LASTEXITCODE。禁止强杀LightDo。
- 发布脚本已先草稿上传/下载逐个SHA核对再公开，失败保留草稿。验收JSON与指纹阻止提前正式tag。源版本1.0.0候选。
- Rust1.99.1/VS17.14.42/SDK26100/NSIS3.12，gh .tools/gh/bin/gh.exe。CARGO_TARGET_DIR=工作区/target/candidate；root app旧，root玻璃fixture最新支持--fullscreen/--static。
