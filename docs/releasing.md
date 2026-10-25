# 开发与发布

当前尚未发布。正式标签被本机玻璃、窗口模式、中文输入和安装验收门槛阻止；不能以编译成功代替。

本机使用 Rust 1.99.1 x86_64-pc-windows-msvc、Visual Studio 2022 Build Tools 17.14.42（C++ 桌面工作负载）及 Windows SDK 10.0.26100。

```powershell
.\scripts\dev.ps1 -Isolated
.\scripts\check.ps1
.\scripts\build.ps1
```

隔离开发数据位于 `local-evidence/dev-data`，不会写入正式用户目录。

运行 `cargo run --release --example glass_background` 打开本项目的动态无隐私测试窗口。它是测试工具，不随应用安装，不作为应用内置背景，也不会在正式运行时启动。

发布者为 Yuan Haoran (Richard-Nido)，无代码签名证书。发布时应如实说明安装包未签名以及 Windows 可能显示发布者验证或 SmartScreen 提示；不得要求用户关闭安全防护。

完整安装器与标签发布入口正在实现。未来发布脚本必须校验干净工作区、版本一致、标签不存在及本机验收提交，再推标签；不得改写已公开标签或绕过失败检查。
