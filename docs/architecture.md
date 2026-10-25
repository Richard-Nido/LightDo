# LightDo 架构与实现决策

## 来源与范围

QuietDo：`be1a46fcb8cf8e18f79649057223e9ab27d1bd3c`，MIT，Copyright (c) 2026 LuvKobe。
Glance：`ec0fb40de5f4d0a8d8edd1442ad5a4d08bacbee2`，MIT，Copyright (c) 2026 lulu-loopp。
实际阅读两者 README、LICENSE、Cargo/构建配置，以及 QuietDo main.js/styles.css/lib.rs/tauri.conf.json/release.yml；Glance panel/widget/push/detector/mouse/screens/settings/fullscreen 与 ui 的 glass/backdrop/scene/wallpaper/render/gfx/window/typing/skins。

QuietDo 的 UI 是透明 WebView 内的 CSS backdrop-filter；这不能证明桌面窗口背景发生折射，且原有 Enter 处理没有 IME 组合保护。因此保留其单清单产品语义，重建 Rust + Win32 + Direct2D/DirectComposition 窗口层，文本编辑交给系统原生编辑控件。不依赖 Electron、Tauri、WebView2 或 Node 运行时。

Glance 的 glass.rs 是系统 HostBackdrop 毛玻璃；skins.rs 的 Glass 才使用圆角边缘位移。scene.rs 通过 DWM 缩略图按 Z 序重建背景，再通过 Windows.Graphics.Capture 取 GPU 纹理；不是固定壁纸，也不靠隐藏主窗口截图。仅提取 scene/backdrop 与位移算法，图形设备、业务模型、窗口管理和 UI 在 LightDo 内独立实现。没有引入监控、传感器、音乐、联网或远程控制依赖。

## 关键技术验证设计

1. 图形设备使用 D3D11 BGRA，多线程上下文保护；D2D 绘制到 DComp 表面。背景与主面板分离，背景采集不包含主面板或场景窗口。圆角距离场产生 RG 位移图，GaussianBlur + DisplacementMap 输出真实背景折射；失效时明确显示降级，不能静默宣称完整玻璃。
2. 单一待办 HWND，原生编辑子控件；边缘无激活 ShowWindow，点击输入再激活。模式转换保留编辑缓冲，几何坐标为物理像素，布局使用 DIP。
3. 使用系统编辑控件保留输入法、选区、粘贴和无障碍支持；组合期间禁止 Enter 提交，处理 IME 结束后的同一次按键与失焦顺序。实际候选窗测试是独立验收门槛。

## 数据与发布边界

计划数据路径 `%APPDATA%\LightDo\state.json`；版本化 JSON，串行同步落盘、临时文件 flush 后原子替换。读取失败保持原文件且禁写，不用空数据覆盖。导入 QuietDo 只复制且由用户显式选择。

当前最低实际测试系统是本机 Windows 11 26H2 build 26300.9711 x64；注册表 ProductName 仍称 Windows 10，不能据此误判系统版本。其他 Windows 11 构建尚未验证。

开发版本先用 0.1.0，完成关键交互和安装验收后才确定首次正式版本。没有签名证书；不会使用上游签名身份。
