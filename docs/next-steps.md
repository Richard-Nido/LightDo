# 当前接续点

完整目标仍为LightDo、本机真实验收、Windows安装包、公开正式Release和最终目录清理。用户已授权推送和正式发布，不重复询问产品需求或发布权限。

## 当前运行状态（2026-10-27）

- 源码main已推送1665d0f，CI 40381163180成功；正式版本候选1.0.0，release-acceptance.json仍pending，尚无正式标签和Release。
- 已安装EXE为%LOCALAPPDATA%/Programs/LightDo/lightdo.exe，SHA-256 c3edc3d2429b8a35d919c56452804527cb9fd0946394a4ed9186db28db60d7de。数据%APPDATA%/LightDo/state.json，四条合成任务。
- Windows刚实际锁定，诊断lockFlag=0/locked=true；已停止所有桌面输入，通过异步问题请求解锁。解锁前不要继续CUA，也不要把锁定界面当作可操作桌面。
- 两个glass_background测试进程均已按准确路径核对后结束；没有强杀LightDo。全屏用例已启动但在激活时发现锁屏，未完成，不可写PASS。
- 偏好已恢复为Edge/Right/Light/48、350×547、Ctrl+Alt+L、自启关闭、pinned=false/locked=false/topmost=false。恢复时正常退出，保留当前全部任务，只原子恢复测试前的设置；任务摘要一致。随后已重启已安装版，保持此实例以检查真实解锁恢复。

## 本轮新增实际验收

- 透明度48→70及恢复48；深色、跟随系统（当前系统暗色）和浅色恢复，设置应用/文件读回正常。
- 固定展开后鼠标/焦点移至测试背景，面板仍显示，phase=Pinned；锁定后标题拖动及右下角缩放均保持矩形；锁定仍可双击行内编辑并Esc退出。
- 真实原生下拉框完成Edge→Floating→Desktop→Edge，同一面板句柄，四条任务保持。Floating非置顶、开启置顶、恢复非置顶的实际窗口属性正确；不适用选项禁用。
- 发现设置窗口原先附属于主面板，Desktop切换会把设置也沉到底层。已改为独立顶层、可任务切换窗口；含修正安装版实际切Desktop后设置仍可见并能操作返回Edge。
- 用户两次确认设置菜单及Ctrl+F10正常。此前助手把工具未列出误判为设置关闭，已经向用户纠正。设置现在可被工具列出，不再需要用户保持打开。
- 自启实际开启后HKCU Run指向已安装EXE并带--startup，关闭后登记消失。未注销/重启Windows，不能据此称登录启动已通过。
- 实际临时注册Ctrl+Alt+Shift+F8制造冲突，设置显示冲突提示，保留旧Ctrl+Alt+L；关闭提示、释放测试注册后新键保存成功，实际从隐藏呼出且foreground=true。
- 恢复默认→应用实际读回Edge/Right/System/48/400×600/400ms/250ms/Ctrl+Alt+L/自启关闭，任务摘要不变。随后已恢复用户测试前偏好（见上文），不遗留测试F8快捷键。
- 32测试中23通过、9真实桌面实验忽略；不要把忽略项记PASS。完整证据verification.md，原始无正文诊断local-evidence内。
- 先前DPI150→100→125→200→150的右边缘布局与中文候选位置已实测；当前系统恢复150%，动画偏好恢复关闭。Win+D新版已有用户手测通过。

## 下一步

1. 等用户解锁，先读本机诊断确认locked=false，再刷新Sky窗口选择。检查同一进程正常呼出/输入恢复。
2. 真实全屏避让：root target中的最新glass_background支持--fullscreen/--static且为整帧绘制，hash56df251e...。它是可见交互测试窗口，启动后需实际前台/边界成立；此前激活失败是锁屏，不能归因产品崩溃。主应用继续用已安装版或candidate，root lightdo.exe仍旧。
3. 剩余真实项目：左右防误触/反向/关联菜单、拖出与重新停靠、剪贴板/键盘删除/多任务滚动、Explorer/锁屏/睡眠恢复、安装GUI/卸载保留及主动删除。不要反复问已确认产品需求。
4. 自启登记清除已测，注销后实际启动未测；本机仅一个物理显示器，不伪造多屏主屏/断开证据。
5. 完成本机关键验收后才finalize-acceptance和release -Publish。发布脚本先草稿上传，重新下载逐一核对SHA后公开，保护真实退出码；本地六项资产、篡改与缺失资产预检已验证，真实发布未执行。
6. 正式下载校验安装之后才执行清理。清理脚本现在会识别并删除旧0.1.0安装包/许可，只保留当前版本；已预览，未执行。最新编辑含scripts/clean-delivery.ps1与check.ps1语法检查，需提交及跟踪CI。

## 桌面工具与安全

使用computer-use的@oai/sky；禁止Windows键、认证/安全/密码管理器/Codex UI，用户物理Esc则停止。所有持久状态统一用globalThis属性引用，避免先前顶层let/const（尤其fs、ss、sw等）遮蔽产生旧值；不要把node fs模块当作Window传入。由list_windows/list_apps返回对象唯一选择，每次用新state.window和截图ID操作。切换窗口后先刷新；UIA下拉set_value可能不触发原生CBN，模式宜点实际下拉项。

不要再用PowerShell FindWindow(class,$null)的false作为窗口不存在证据：字符串空值绑定可能不等于原生NULL。只读EnumWindows按本应用PID过滤曾确认设置真实可见；不得用它绕过Sky注入输入或构造未由Sky返回的Window。

Rust1.99.1 MSVC/VS17.14.42/SDK10.0.26100/NSIS3.12，gh在.tools/gh/bin/gh.exe。构建CARGO_TARGET_DIR=工作区/target/candidate；脚本遵守该路径并核对版本。当前EXE依赖检查627200字节、未签名，直接导入只见Windows系统DLL，未见VC运行库DLL。

锁屏等待期间已新增scripts/test-clean-delivery.ps1并接入check.ps1：隔离合成Git仓库的脏树/已跟踪文件拒绝及真实合成文件删除/保留均通过；真实项目仍未清理。清理旧版本文件、根/父目录链接保护、CRLF版本识别已补，只有最终交付完成后才对实际项目执行。未来不必为此重复创建无意义文档CI循环。
