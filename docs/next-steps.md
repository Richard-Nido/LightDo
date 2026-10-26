# 当前接续点

目标仍为完整LightDo、本机真实验收、Windows x64安装包、公开正式Release及最终目录清理。用户已授权发布，确认使用Richard-Nido/LightDo，不重复询问需求或推送权限。

## 当前事实

- 版本1.0.0已准备，尚未正式发布；release-acceptance.json保持pending，所有门槛仍false。
- GitHub Richard-Nido登录有效；cf8de57的CI 40296556997成功。之后右边缘/动画/资源改动需提交、推送与跟踪新CI。
- 当前实际使用安装路径 `%LOCALAPPDATA%\Programs\LightDo\lightdo.exe`，正式数据目录 `%APPDATA%\LightDo`；升级测试从隔离数据复制了四条合成任务。不要再误认运行的是旧隔离实例。
- 实际0.1.0→1.0.0升级，状态文件字节与新payload匹配通过；保留安装供GUI验收。同版本后续覆盖也核对数据未变。
- 用户允许恢复自动桌面验收。用computer-use @oai/sky，按返回窗口选择，禁止Windows键、Codex/认证/安全UI和替代输入通道。用户物理Esc则停止输入。Win+D新版往返已有用户手测通过。
- 输入、标题、行布局和菜单要求见product-decisions.md；当前安装已实际行内提交、长文本完成/撤销、缩小重排、150%拼音Space选词/Enter新增、浅色玻璃。未观察上述操作抖动，不夸大为所有偶发问题消失。
- 默认/当前右侧。边缘条计入交互区，Esc手动关闭须离开再进入；托盘隐藏单独状态停用检测；回访用户启动自然收回。呼出先在隐藏状态预备真实背景和合成表面，再滑入，目标220ms，遵守系统减少动画。
- 普通最大化标题窗不再误判全屏。真实右边缘鼠标展开、留边不抖、离开收回、CtrlAltL和Esc同PID均已观察。
- 系统动画原为关闭，已临时通过设置打开并观察实际滑入，诊断left递进2568→2023；结束已恢复关闭。不要只凭代码声称60fps或反向手势均通过。
- 捕获池隐藏时Scene.rest复用，停止场景处理/读取，不每次销毁重建；5秒后释放镜头及表面并DXGI Trim。诊断区别captureAllocated与captureActive，隐藏应active=false，允许池仍allocated。
- 同进程10次展开/收回后约49秒：CPU0.0032%、GPU0%、工作集平均95.50MiB、私有70.77MiB、共享GPU74.75MiB，47样本隐藏。哈希058023...为该10次构建；后来又加计时器仅变更才重设及64ms长帧钳制，当前安装哈希ac50a954...，需对应其最新测量，不混用。
- 旧GDI测试背景会被捕获到纯色/半绘制帧，曾导致第8次短暂纯色面板。例程已改为内存整帧BitBlt；当前新例程位于target/candidate/.../examples，不要重启旧root例程。仍需复核首帧与闪烁，不能据此虚报完全解决。
- 当前32项测试：23通过、9忽略；原生捕获专项及60秒压力证据见verification.md。先前夹具DWM cloak失败已定位，未采用丢首帧/会话重建实验。

## 下一步

1. 完成当前右边缘最新安装构建隐藏采样，更新performance.md；继续真实动画/玻璃首帧、键盘与设置/模式交互。
2. 实际DPI100/125/200%、Explorer/锁屏/睡眠、全屏、拖出重停靠等；本机单屏不能伪造多屏证据。
3. 继续安装GUI及主动删除/自启清理，完成最终候选全部关键回归。
4. 只有真实门槛通过才finalize-acceptance，然后release -Publish，跟踪Actions、下载校验及安装。用户已授权，无需再次询问发布。
5. 验收和正式Release完成后实际clean-delivery；目前只预览过，不要声称工作目录已清理。

## 工具

Rust1.99.1 MSVC、VS17.14.42、SDK10.0.26100、NSIS3.12，gh位于.tools/gh/bin/gh.exe。构建使用CARGO_TARGET_DIR=工作区/target/candidate，scripts/build.ps1与package.ps1遵守该路径并检查资源版本。不要拿旧root二进制验收最新源码。

正常--shutdown退出LightDo，不强杀。只在得到真实session_id时轮询；不要重启仍活进程代替等待。保留用户数据，清理严格限定工作区内已核对路径。
