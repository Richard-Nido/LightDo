# 当前接续点

完整目标仍为LightDo、本机真实验收、Windows安装包、公开正式Release和最终目录清理。用户已授权推送和正式发布。尚无正式标签/Release；验收JSON仍pending。

## 当前现场（2026-10-27，任务保留补强后）

- 用户已确认“睡眠恢复，组件输入，桌面图标都正常”。同时报告退出重开任务不可见；原AppData文件仍含5条未完成任务，旧版再次启动能显示，原先不可见触发原因尚未精确确认。
- 已补强Known Folder固定数据目录、500ms有界共享锁重试、失败明确空态和禁写保护；设置通用页新增打开真实数据文件夹，已实际打开Explorer并看到唯一state.json。
- 最新已安装EXE SHA为24fdab846550463d90189f1471e20c3c359e7be6de5d2368cfe585e7cf55cf5b，PID22844，08:39:45启动。33项中24通过、9忽略；候选安装器370878字节。尚无正式Release。
- 三次实际独立冷启动PID20416/35776/22844，从TEMP工作目录均读到5条未完成任务，文件SHA不变；第三次还实际观察到五条任务显示。test-restart.ps1的首次诊断管道错误已修正，不把诊断子进程管道错误当主应用崩溃。
- 当前Edge/Right/Light/48、350×547、Ctrl+Alt+L，自动隐藏/全屏避让开启，自启/锁定/固定展开关闭。已实际设置恢复Right，五条任务保留。
- 新构建日常右边缘隐藏44.29秒，42个样本均隐藏，CPU0.0106%、GPU0%、工作集106.14MiB、私有73.19MiB、共享GPU88.92MiB，646句柄。之前打开过设置与系统数据目录，不与旧样本直接归因比较。
- 为保护用户原任务，local-evidence/restart-data-protection/state-before-shutdown.json有临时验收前副本；当前原AppData完整，最终清理前以当前AppData为权威，不能用旧副本覆盖后续编辑。该副本未提交Git。
- 无隐私背景PID33684仍在；轻待隐藏，已关闭设置和打开的数据目录窗口。后续GUI前检查锁状态并刷新Sky。

## 上一批修复记录（以下旧PID和等待状态仅供历史定位）

- 用户解锁后，PID35024保持且实际呼出、输入与保存成功，边缘模式恢复已通过；正式AppData现在五条合成任务。
- 之后发现并修复两个实际问题：左侧置顶面板遮挡独立设置；快速拖动时队列消费采样起点太晚。修正已检查、打包、安装并实际拖出Floating再拖回Edge，设置在Edge和Desktop均可操作。
- 当前已安装1.0.0 SHA-256为0751214fb061fb5f8599312f13f5a8a2d96fc8d7edcf547c489453b4cb1ae945，PID36444，2026-10-27 08:18:18启动。安装路径%LOCALAPPDATA%/Programs/LightDo；数据%APPDATA%/LightDo/state.json。
- 当前主动停留Desktop模式，Right偏好、Light/48、350×547、Ctrl+Alt+L、autostart=false、pinned=false/locked=false、fullscreen_pause=true。桌面左侧位置来自拖回左侧实测，回Edge会按Right定位。
- 已关闭测试背景和设置窗口，向用户异步请求“实际睡眠、唤醒解锁、Desktop组件输入与桌面图标”结果。用户操作期间停止桌面输入；可做源码/CI工作。收到回复后先诊断锁定状态、刷新Sky选择。
- 32项中23通过、9忽略；本轮check及打包成功，当前候选安装器368722字节，尚非最终公开下载资产。

## 本轮真实新增结果

- 无边框全屏受控背景，鼠标停留右边缘仍隐藏；快捷键主动呼出且foreground=true、背景就绪，可输入。
- c3edc3d2构建解锁/使用后右边缘隐藏50.78秒：47个窗口样本均隐藏，CPU0.0046%、GPU0%，工作集92.21MiB、私有71.22MiB、共享GPU102.11MiB。不能套用为后续修复哈希的实测。
- 左侧实际非激活展开[12,390,537,1210]；移开收回；此左侧测试临时关闭全屏避让，测试后已恢复。
- 最新修正版实际拖出[185,390,710,1210] Floating/topmost=false，再拖回[12,390,537,1210] Edge/topmost=true，同一面板/五条任务，docs/images/left-redock-current.png。
- 菜单实际打开后正常--shutdown在0.352秒内结束，任务文件哈希不变（cd858b3d构建）。最初沿用PowerShell旧LASTEXITCODE误报，正确Start-Process -Wait复测通过，不是产品退出失败。
- 两轮同版本覆盖安装均保留状态逐字节相同，新安装重启恢复五条任务。

## 继续完成

1. 睡眠/组件输入/桌面图标已由用户手测通过，不再重复询问。继续剩余真实交互和安装；实际自动操作前检查锁状态。
2. 剩余真实交互：当前构建选区/复制连续粘贴/空白/删除/多任务滚动，快速掠过/返回/反向/关联菜单，Explorer重启和Desktop恢复。不要重问已确认产品需求或已测菜单/快捷键。
3. 完整安装GUI/主动删除数据及卸载自启清理。操作之前保护正式AppData五条合成任务；GUI删除/安装遵循computer-use的动作时确认规则，非UI已授权安装不额外询问。
4. 最新构建可见静态与边缘隐藏性能、启动时间，最终关键交互回归。单物理显示器不伪造多屏；本机只测Win11 26H2。
5. 完成关键验收后才finalize-acceptance并release -Publish；标签Actions先构建检查，草稿上传并下载校验所有六项资产后公开。之后最终下载校验安装，实际清理。现在不能写“全部完成”。

## 工具与保护

- Sky通过mcp__node_repl__js的@oai/sky。禁止Windows键、Codex/认证/安全UI；物理Esc或锁屏立即停止输入。
- 所有持久变量使用globalThis读写，避免旧顶层fs/sw/ss等遮蔽。从list_windows返回对象唯一选择，每次用state.window更新；切窗口后刷新。原生下拉优先实际点击，set_value可能不触发CBN。
- get_window_state有时UIA树落后一帧，原生页切换后重新观察；设置模式切换后截图可能包含主面板区域和第二张设置截图，不能误判设置消失。
- 系统原生GUI EXE在PowerShell中不要沿用旧LASTEXITCODE；需要退出码时Start-Process -Wait -PassThru，读取ExitCode。禁止强杀LightDo；正常--shutdown后等待。
- 不用PowerShell FindWindow(class,$null)断言窗口不存在。需要只读诊断时过滤本项目PID。
- Rust1.99.1/VS17.14.42/SDK10.0.26100/NSIS3.12；gh在.tools/gh/bin/gh.exe。构建CARGO_TARGET_DIR=工作区/target/candidate。root app仍旧；root玻璃背景fixture最新、支持--fullscreen/--static、双缓冲整帧绘制。
- 清理脚本隔离合成测试和CI已通过，本项目实际清理未执行。仅在最终Release下载安装验证后运行，保留源码、Git、正式文档、当前版本交付资产及项目外用户数据。
