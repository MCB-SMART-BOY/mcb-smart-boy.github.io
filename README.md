# MCB / LOG

MCB 的个人网站，记录 Rust、Linux、系统工具与形式化验证方面的学习和实践，也介绍正在维护的开源项目与 CodeFlow·代码灵动社团。

[访问网站](https://mcb-smart-boy.github.io/) · [阅读文章](https://mcb-smart-boy.github.io/writing/) · [GitHub](https://github.com/MCB-SMART-BOY)

## 这里有什么

- **个人与项目**：关注方向、公开项目、教育与技能、社团介绍，以及放在首页末尾的“关于”。
- **文章与笔记**：首页展示最新三篇，文章归档保留完整内容；可以按目录浏览，也可以用文章目录跳转到具体段落。
- **阅读体验**：浅色暖纸与深色主题、中英文界面、可调整的侧栏和移动端导航。文章保留中文原文，界面切换不等于文章翻译。

部分文章用于演示阅读流程。网站没有登录、评论或在线编辑功能，内容直接在仓库里维护。

## 在本地运行

需要先安装 [Rustup](https://rustup.rs/)，并准备本机 Rust/C++ 编译依赖和可用网络。仓库会使用 `rust-toolchain.toml` 指定的 nightly 工具链。

首次运行，在仓库根目录执行：

```sh
cargo xtask setup
cargo xtask dev
```

打开 **http://127.0.0.1:3000**。开发服务会监听源码和文章变化，重新构建后刷新页面；停止服务使用 `Ctrl+C`。后续开发只需运行 `cargo xtask dev`。

`setup` 会安装所需组件和构建工具，工具位于用户共享目录；已有版本不符合要求时可能被替换，其他项目也可能受影响。日常开发和构建不会自动安装或升级工具。完整说明见[维护指南](.omp/maintenance.md)。

如果只想构建一次、再启动服务：

```sh
cargo xtask build
cargo run --locked
```

修改页面或内容后，需要重新构建。单独运行 `cargo run` 不会更新浏览器资源；只运行静态导出也不会更新这套开发资源。

## 写作

文章放在 `content/posts/`。在已有目录中新建 `.md` 文件，补齐开头的元数据，再写正文，例如：

```markdown
---
title: 一篇新的笔记
date: "2026-10-14"
summary: 这篇笔记要讨论的问题。
reading_minutes: 3
---

## 从一个问题开始

这里写正文。
```

新建目录时，添加一个带有 `title` 的 `_index.md`，作为这个目录的介绍。文件位置决定文章网址，`weight` 可以调整目录内的排列顺序；首页的最新三篇始终按日期从新到旧选取。文章不需要在 Rust 代码中另行登记。

字段、图片、文章间链接以及移动文件时的注意事项，见[内容规范](.omp/content-format.md)。

## 构建与发布

生成可部署的静态网站：

```sh
cargo xtask export
```

输出位于 `run/pages/`，包含完整 HTML、样式、浏览器脚本、WASM、图片和字体。网站使用 Rust / Leptos 构建，Axum 负责本地开发；部署到 GitHub Pages 后不需要运行后端服务。未加载 JavaScript 或 WASM 时，正文和普通链接仍可使用。

运行项目完整检查：

```sh
cargo xtask check
```

检查涵盖格式、Clippy、浏览器端编译、开发构建、测试和静态导出。`run/` 是可重新生成的产物目录，不需要提交或手工修改。

本仓库通过 GitHub Actions 发布：**推送到 `main` 会触发网站更新**，构建检查和 Pages 环境门禁通过后部署；也可以手动选择本仓库中已推送的提交。具体步骤、工具链限制和恢复方式见[维护指南](.omp/maintenance.md)。

## 深入了解

- [网站设计](.omp/design.md)：页面组织、配色、布局、动效和交互。
- [技术架构](.omp/architecture.md)：内容索引、静态生成与浏览器接管。
- [内容规范](.omp/content-format.md)：目录、文章格式和链接规则。
- [维护指南](.omp/maintenance.md)：开发、构建、发布与故障排查。
- [验证指南](.omp/verification.md)：交互验收场景和检查边界。

字体在本地托管，来源与许可见 [public/fonts/NOTICE.txt](public/fonts/NOTICE.txt)。这个仓库维护网站；GitHub 个人主页的介绍和视觉资源由独立的 [MCB-SMART-BOY 仓库](https://github.com/MCB-SMART-BOY/MCB-SMART-BOY) 维护。
