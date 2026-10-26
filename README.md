# 轻待 LightDo

Rust + 原生 Windows 的纯本地单清单待办，目标平台 Windows 11 x64。

**正在开发与技术验证中，尚无正式 Release。** 本机已生成未签名的 0.1.0 开发安装包；编译产物没有提交到仓库，当前尚未提供公开安装包下载。源码可构建和安装器冒烟测试不代表完整产品已通过验收。进度见 [实施清单](docs/implementation-checklist.md)；真实结果见 [验收记录](docs/verification.md)。

当前源码包含待办、原子存储保护、原生文本编辑、实时背景玻璃、边缘状态机、悬浮/桌面层级、托盘、设置与安装发布流程；新增行为仍在验收中。输入法的早期基线运行证据：

![实际拼音候选窗，受控动态网格背景](docs/images/p0-ime-inline.jpg)

背景通过 DWM 缩略图与 Windows.Graphics.Capture 在本机内存中合成，再用 Direct2D 模糊与位移采样。没有使用固定壁纸或 CSS blur 替代桌面折射。完整视觉与模式验收仍在进行。

## 本地开发

安装 Rust 1.99.1 MSVC、Visual Studio 2022 Build Tools 的 C++ 桌面工作负载和 Windows SDK。

```powershell
cargo build --locked
cargo test --locked --lib
cargo run --locked
```

右键面板打开模式与外观菜单。`Ctrl+Alt+L` 呼出并输入；双击任务编辑，Enter 提交，Esc 取消。数据位于 `%APPDATA%\LightDo\state.json`；开发验收可用环境变量 `LIGHTDO_DATA_DIR` 指向隔离目录。轻待使用独立的数据目录，不提供其他应用的数据导入。

菜单已按「窗口模式 / 外观 / 位置与行为」分组，设置分为「外观 / 窗口 / 通用」三页。实际运行截图和验证范围见 [菜单与设置改版](docs/menu-redesign.md)。

本机资源占用的实际数值、测量方法和优化状态见 [资源占用记录](docs/performance.md)。

完整开发检查和 EXE 打包入口见 [releasing.md](docs/releasing.md)。正式标签自动发布流程已配置，但本机验收记录仍为 pending，当前不能发布正式版本。桌面已经解锁，真实交互验收继续进行。

架构与来源见 [architecture.md](docs/architecture.md)；许可见 [LICENSE](LICENSE) 和 [第三方声明](THIRD_PARTY_NOTICES.md)。
