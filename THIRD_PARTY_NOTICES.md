# 上游与第三方许可

LightDo 由 Yuan Haoran (Richard-Nido) 独立开发和发布，不继承上游签名或发布身份。

## Glance — MIT

Copyright (c) 2026 lulu-loopp。源码来源：<https://github.com/lulu-loopp/glance>，提交 `ec0fb40de5f4d0a8d8edd1442ad5a4d08bacbee2`。

实际复用：`src/ui/scene.rs`、`src/ui/backdrop.rs`，保留背景缩略图重建、Windows.Graphics.Capture、GPU 纹理交接与相关测试；将跨模块日志及屏幕辅助函数适配到 LightDo。`src/ui/gfx.rs` 的 `displaced` 从 `ui/skins.rs` 的圆角距离场位移算法改编；`src/platform.rs` 的桌面宿主查找和 Z 序维护从 `ui/window.rs` 适配。没有包含 Glance 的业务模块、品牌图标或嵌入字体。

完整许可见 [docs/LICENSE-Glance](docs/LICENSE-Glance)。

## QuietDo — MIT

Copyright (c) 2026 LuvKobe。源码来源：<https://github.com/LuvKobe/quietdo>，提交 `be1a46fcb8cf8e18f79649057223e9ab27d1bd3c`。

保留其产品语义：单清单、顶部新增、完成保留原位、双击编辑、即时删除。LightDo 重写窗口、图形、输入和存储，没有打包 QuietDo 的前端、Tauri 后端、图标或依赖。出于来源追踪一并保留完整许可：[docs/LICENSE-QuietDo](docs/LICENSE-QuietDo)。

## Rust 依赖

精确解析版本以 Cargo.lock 为准。运行依赖限于 serde/serde_json、Microsoft windows/windows-core/windows-numerics 及其支持 crate。构建依赖 embed-resource 及其解析和 MSVC 工具发现依赖。发布前由 `cargo metadata --locked` 生成依赖许可清单，随安装包提供；不打包编译器或 Windows SDK。
