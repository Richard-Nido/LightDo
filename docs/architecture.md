# LightDo 架构与实现决策

## 来源与范围

QuietDo：`be1a46fcb8cf8e18f79649057223e9ab27d1bd3c`，MIT，Copyright (c) 2026 LuvKobe。
Glance：`ec0fb40de5f4d0a8d8edd1442ad5a4d08bacbee2`，MIT，Copyright (c) 2026 lulu-loopp。
实际阅读两者 README、LICENSE、Cargo/构建配置，以及 QuietDo main.js/styles.css/lib.rs/tauri.conf.json/release.yml；Glance panel/widget/push/detector/mouse/screens/settings/fullscreen 与 ui 的 glass/backdrop/scene/wallpaper/render/gfx/window/typing/skins。

QuietDo 的 UI 是透明 WebView 内的 CSS backdrop-filter；这不能证明桌面窗口背景发生折射，且原有 Enter 处理没有 IME 组合保护。因此保留其单清单产品语义，重建 Rust + Win32 + Direct2D/DirectComposition 窗口层，文本编辑交给系统原生编辑控件。不依赖 Electron、Tauri、WebView2 或 Node 运行时。

Glance 的 glass.rs 是系统 HostBackdrop 毛玻璃；skins.rs 的 Glass 才使用圆角边缘位移。scene.rs 通过 DWM 缩略图按 Z 序重建背景，再通过 Windows.Graphics.Capture 取 GPU 纹理；不是固定壁纸，也不靠隐藏主窗口截图。仅提取 scene/backdrop 与位移算法，图形设备、业务模型、窗口管理和 UI 在 LightDo 内独立实现。没有引入监控、传感器、音乐、联网或远程控制依赖。

## 关键技术验证设计

1. 图形设备使用 D3D11 BGRA，多线程上下文保护；D2D 绘制到 DComp 表面。背景与主面板分离，背景采集不包含主面板或场景窗口。圆角距离场产生 RG 位移图，GaussianBlur + DisplacementMap 输出真实背景折射；失效时明确显示降级，不能静默宣称完整玻璃。
2. 单一待办 HWND，原生编辑子控件；边缘无激活 ShowWindow，点击输入再激活。模式转换保留编辑缓冲，几何坐标为物理像素，布局使用 DIP。移动/缩放使用普通消息循环中的捕获手势，避免系统模态移动循环暂停背景刷新。
3. 使用系统编辑控件保留输入法、选区、粘贴和无障碍支持；组合期间禁止 Enter 提交，处理 IME 结束后的同一次按键与失焦顺序。实际候选窗测试是独立验收门槛。

## 数据与发布边界

数据路径 `%APPDATA%\LightDo\state.json`；版本化 JSON，串行同步落盘、临时文件 flush 后原子替换。读取失败保持原文件且禁写，不用空数据覆盖。按用户后续要求，不提供 QuietDo 或其他应用的数据导入，见 product-decisions.md。

当前最低实际测试系统是本机 Windows 11 26H2 build 26300.9711 x64；注册表 ProductName 仍称 Windows 10，不能据此误判系统版本。其他 Windows 11 构建尚未验证。

## 桌面层级方案的实验与修正

最初的 SetParent 桌面子窗口方案已经否决。受控白色宿主与红色子窗实验中，在子窗成为子窗口之前设置 WDA_EXCLUDEFROMCAPTURE，也没有使其从宿主缩略图中消失：两组红色区域亮度均为 0.21176487。这个反例说明不能据此假设桌面采样排除了自身。实验保留在忽略运行的 `embedded_child_capture_exclusion_probe` 中；首次探索结果为 FAIL，不能包装成通过。

当前采用 Glance ui/window.rs 的桌面层级管理思路：待办仍为顶层 HWND，切为工具窗并持续维护在桌面宿主上方、普通应用下方，随桌面 Z 序变化显示。编辑时允许临时取得焦点，失焦后回到底层；不同于单纯取消置顶。这个结构使 scene 可以按真实 Z 序采样桌面并排除主面板。Win+D 往返、图标交互、第三方桌面外观兼容和 Explorer 重启仍须在解锁后实际验收，未宣称通过。

隐藏的控制窗口独立承载托盘、快捷键、单实例唤起和会话通知。图形设备失效时释放采集/表面并重建，系统锁定或挂起时暂停捕获。显示透明效果关闭、高对比度、远程会话、软件渲染的明确降级状态，不把降级材质当作完整折射。

开发版本先用 0.1.0，完成关键交互和安装验收后才确定首次正式版本。没有签名证书；不会使用上游签名身份。
