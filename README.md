# MCB / LOG — 个人网站

[访问网站](https://mcb-smart-boy.github.io/) · [GitHub 个人主页](https://github.com/MCB-SMART-BOY)

这里维护 MCB 的个人网站源码、公开资料和文章内容。网站使用 Rust / Leptos，在构建时生成完整静态 HTML，由 Rust/WASM 接管浏览器交互；GitHub Pages 不运行后端服务。

当前三篇文章为示例内容，不代表作者已发布成果。GitHub 个人主页的介绍与视觉资源由独立的 [MCB-SMART-BOY 仓库](https://github.com/MCB-SMART-BOY/MCB-SMART-BOY) 维护。

## 本地开发

需要系统已有 `rustup`、Rust/C++ 编译依赖及可用网络。仓库的 `rust-toolchain.toml` 选择滚动的 nightly（含 `rustfmt`、`clippy` 和 `wasm32-unknown-unknown`）；在仓库根目录首次准备后启动开发监听：

```sh
cargo xtask setup
cargo xtask dev
```

`dev` 由 cargo-leptos 自动构建、提供本地服务并监听变更；打开 `http://127.0.0.1:3000`。监听进程仍在运行不代表每次重新编译都成功，须查看构建输出。也可以执行 `cargo xtask build` 单次生成 `run/site/`，之后用 `cargo run --locked` 启动本地 SSR 服务；单独运行 `cargo run` 不会构建浏览器资源。普通 `cargo test --locked` 仍可运行。

工具由 `cargo xtask setup` 通过标准 Rustup/Cargo 安装在用户共享目录，不再隔离到仓库 `run/`。它检查所需版本并在版本不符时更新；日常 `build`、`dev`、`export`、`check` 只检查，不主动安装。`cargo-leptos` 0.3.11、`wasm-bindgen-cli` 0.2.129 和 `wasm-opt` 0.116.1 各自按其发行包的锁文件安装，网站的 `Cargo.lock` 只锁网站与 workspace 依赖，不锁全局工具；共享工具升级可能影响其他项目。滚动 nightly 后续可能改变，同一源码 SHA 不保证复现相同编译器或产物，发布运行记录实际版本。

## 静态导出与发布

`cargo xtask export` 单次 release 构建并用同次构建的原生二进制导出完整网站到 `run/pages/`；`cargo xtask check` 检查 WASM、SSR/WASM、workspace 测试、release 导出和关键页面文件。`run/site/` 是开发资源，`run/site-release/` 是发布构建资源，`run/pages/` 是完整静态产物；这些目录均为生成产物，不手工修改或提交；整个 `run/` 可随时删除回收空间，`cargo xtask build` / `export` / `check` 会按需重建（普通 `cargo run` 不重建，须先构建），2026-10-11 已清理旧 dev.sh 隔离缓存。

`.github/workflows/pages.yml` 仅支持手动发布：输入本网站仓库中已推送的完整、小写 40 位提交 SHA，验证后部署同次构建的 Pages artifact。不要输入个人主页仓库的旧 SHA；推送不会自动发布。CI 准备 nightly、执行 `cargo xtask setup` 和 `cargo xtask check`，另存源码 SHA 与实际工具版本 provenance，不混入站点。提交、推送与发布需要分别授权。

## 维护入口

- [项目上下文与验证流程](.omp/AGENTS.md)
- [目录与文章内容规范](.omp/content-format.md)

页面行为 smoke 后运行 `cargo xtask check`；本地还需独立运行 `"$HOME/.omp/agent/validate.sh"`。本地验收不能代替 GitHub Pages 上线后的平台验收。
