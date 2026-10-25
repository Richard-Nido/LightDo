# 轻待 LightDo

Rust + 原生 Windows 的纯本地单清单待办，目标平台 Windows 11 x64。

**正在开发与技术验证中，尚无正式 Release 或可用安装包。** 源码可构建不代表完整产品已通过验收。进度见 [实施清单](docs/implementation-checklist.md)；真实结果见 [验收记录](docs/verification.md)。

当前已实现待办模型、原子存储保护、原生文本输入、实时背景合成与圆角边缘位移，以及边缘/悬浮/桌面窗口层的初步实现。输入法运行证据：

![实际拼音候选窗，受控动态网格背景](docs/images/p0-ime-inline.jpg)

背景通过 DWM 缩略图与 Windows.Graphics.Capture 在本机内存中合成，再用 Direct2D 模糊与位移采样。没有使用固定壁纸或 CSS blur 替代桌面折射。完整视觉与模式验收仍在进行。

## 本地开发

安装 Rust 1.99.1 MSVC、Visual Studio 2022 Build Tools 的 C++ 桌面工作负载和 Windows SDK。

```powershell
cargo build --locked
cargo test --locked --lib
cargo run --locked
```

右键面板打开模式与外观菜单。`Ctrl+Alt+L` 呼出并输入；双击任务编辑，Enter 提交，Esc 取消。数据位于 `%APPDATA%\LightDo\state.json`；开发验收可用环境变量 `LIGHTDO_DATA_DIR` 指向隔离目录，不会自动读取或修改 QuietDo 数据。

架构与来源见 [architecture.md](docs/architecture.md)；许可见 [LICENSE](LICENSE) 和 [第三方声明](THIRD_PARTY_NOTICES.md)。
