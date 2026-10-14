# 网站维护、构建与发布

本文记录网站仓库 `MCB-SMART-BOY/mcb-smart-boy.github.io` 的工具链、产物、发布来源和历史风险。以下命令均在网站仓库根目录执行；配置以 [`rust-toolchain.toml`](../rust-toolchain.toml)、[`Cargo.toml`](../Cargo.toml)、[`.cargo/config.toml`](../.cargo/config.toml)、[`xtask/src/main.rs`](../xtask/src/main.rs)、[`xtask/src/tools.rs`](../xtask/src/tools.rs) 和 [Pages workflow](../.github/workflows/pages.yml) 为准。本文的历史日期、run 和浏览器观察不是本轮重新验收或当前线上状态；行为与质量门的详细矩阵见 [verification.md](verification.md)。

## 归属与边界

- 本仓库拥有完整网站源码、文章和 Pages 发布；独立 GitHub 个人主页仓库 `MCB-SMART-BOY/MCB-SMART-BOY` 的简介、视觉与 `profile-*.svg` 不在此同步。个人主页变更须读取该仓库自己的约束，不从本站自动复制教育、经历、社团、荣誉或项目资料。
- 线上地址为 <https://mcb-smart-boy.github.io/>；Pages source 应设置为 GitHub Actions。本仓库唯一 Pages 发布配置为 [`.github/workflows/pages.yml`](../.github/workflows/pages.yml)，不安装旧的 `.omp/pages/workflow.yml` 副本。变更远端 Pages 设置须单独授权，不覆盖未识别的远端内容。构建时由 Axum/Leptos SSR 生成静态 HTML，线上不运行 Axum。
- `run/` 与 `.env`、`.env.*` 被 [`.gitignore`](../.gitignore) 忽略；忽略不是权限控制。不要把构建产物、凭据或本地设置写入源码或发布包，也不要手工修改 `run/pages/` 充当发布输入。提交、推送、部署、回滚均需针对实际操作和影响取得授权；尤其推送 `main` 会触发自动发布。

## 首次准备与日常开发

系统须已有 `rustup`、Rust/C++ 编译依赖和可用网络。根 toolchain 文件选滚动 `nightly`、`minimal` profile、`rustfmt`／`clippy` 及 `wasm32-unknown-unknown` target。首次准备在知悉共享工具替换影响后执行：

```sh
cargo xtask setup
```

`setup` 补齐 nightly 组件与 target，核对 PATH 上的 `cargo-leptos` 0.3.11、`wasm-bindgen`（发行包 `wasm-bindgen-cli`）0.2.129 和 `wasm-opt`（发行包 `wasm-opt`）0.116.1；版本不符时以 Cargo `install --locked --force` 替换并复核。工具装在用户共享的 Rustup/Cargo 路径，**不是**项目私有 `run/`：会影响其他项目，因此不要把 `setup` 放进日常命令的隐式前置步骤。三个工具分别按各自发行包锁文件安装，网站 [`Cargo.lock`](../Cargo.lock) 只锁本站与 workspace 的依赖图，不锁共享安装工具或持续滚动的 nightly。`cargo-leptos` 安装启用 `no_downloads` feature；`wasm-opt` 用 `--no-default-features` 排除不需要的可选 LLVM/DWARF 调试支持，仍是执行真实 Binaryen 116 WASM 优化的工具，不能省略优化器冒充 release 构建。安装耗时、占用与网络、编译环境相关；同一源码 SHA 日后不保证相同编译器与产物，发布时记录实测版本。

开发自动构建、提供本地服务并监听改动：

```sh
cargo xtask dev
```

浏览器打开 <http://127.0.0.1:3000/>。监听进程继续运行不证明每次修改都编译成功，须观察每次构建输出。需要单次构建与独立本地 SSR 服务时，依次执行：

```sh
cargo xtask build
cargo run --locked
```

`build` 为开发资源生成 `run/site/`；普通 `cargo run --locked` **不会**生成浏览器 JS/WASM，单独运行它不足以证明交互正常。可用 `BLOG_ADDR=127.0.0.1:4000 cargo run --locked` 改端口；结束服务用 `Ctrl+C`。默认仅监听回环地址，未设计 TLS 和部署安全边界前不要公开暴露本地服务。`cargo xtask` alias 可从 workspace 成员目录调用，任务内部始终以仓库根运行；普通 Cargo/Rustup 命令也使用标准工具链与用户配置，不重写 `CARGO_HOME`、`RUSTUP_HOME`、`CARGO_TARGET_DIR`、`XDG_*`、`TMPDIR`、`TRIVY_CACHE_DIR`。

`LEPTOS_OUTPUT_NAME` 错配可能使 `cargo run` 请求不存在的 WASM 文件；`Cargo.toml` 的开发 `debug = 1` 对应曾出现的链接器调试节溢出，而 `disable-erase-components = true` 关系到 SSR/WASM hydration 一致性。排查时不要将这些设置当作共享工具安装选项或随意关闭；原因、症状和架构约束见 [architecture.md](architecture.md#ssr-初值颜色与-hydration)。

## 静态导出、产物与资源排查

首次准备完成后单次 release 构建、无 HTTP 监听地导出：

```sh
cargo xtask export
```

`export` 先检查已安装的工具版本（不会自行 `setup`），再以独立 `run/site-release/` 生成带哈希名的 SSR/WASM、CSS 和资源，仅构建成功后运行**同一次 cargo-leptos 构建**的原生二进制 `export`，生成完整静态站点 `run/pages/`。不要另外以不同配置重建导出二进制；失败时不能拿旧导出冒充本次结果。`cargo xtask check` 同样不隐式安装：执行 Rust format、clippy、WASM hydrate check、开发 SSR/WASM 构建、锁定 workspace 测试、release 导出与首页／404／归档关键文件核验，详细门见 [verification.md](verification.md)。

| 位置 | 职责与生命周期 |
|---|---|
| `run/site/` | 开发 SSR 配套资源；`cargo xtask build`／`dev` 生成，普通 `cargo run` 只读取。 |
| `run/site-release/` | release 构建的资源输入，必须与本次导出二进制匹配，不混作本地开发资源。 |
| `run/pages/` | 成功导出的完整静态站点，由 exporter 按所有权清单接管和替换；供纯静态预览及 CI Pages artifact 使用。 |
| `run/provenance/` | workflow 单独写入源码 SHA 和版本的 CI 临时记录，并作为非网站 artifact 上传；`cargo xtask check` 不会生成它。不能假定本地清理 `run/` 会删除平台上已上传的 artifact。 |

Cargo 二进制与 `hash.txt` 位于 `cargo metadata --locked --no-deps --format-version 1` 提供的 `target_directory` 对应 host/release 位置；xtask 同时读取 `rustc -vV` 的 host triple，尊重用户 `CARGO_TARGET_DIR` 等设置。不要硬编码 `target/release`，更不要使用已删除的旧 `run/build/release/`。`run/` 整体可再生，但删除会丢失本地尚需核对的产物；工具就绪且构建成功后 `build`／`export`／`check` 才能按需重建对应部分，普通 `cargo run` 不会重建它们。旧 `dev.sh` 的隔离缓存删除只记在下方历史记录，不是现在要执行的清理命令。

导出沿用生成索引，产出规范路径的 `index.html`、独立 `404.html` 和本地哈希资源；不手写路由清单，也不要向 `run/pages/` 手动添改文件。导出须核对本次资源与现有输出所有权，在临时目录组装再替换；准备失败保留旧输出，替换失败会尝试恢复，但不能承诺任何文件系统故障都自动回滚。具体校验、可信输入边界及失败时备份处理见 [architecture.md](architecture.md#导出完整性归属与失败语义)。

出现“SSR 可见但浏览器交互或动效不对”时，先区分本地 SSR 所用 `run/site/` 和纯静态导出 `run/pages/`／`run/site-release/`，核对最近一次相应构建是否成功、浏览器实际加载的哈希资源与 HTML 引用是否匹配、WASM 请求和 hydration 控制台错误；另核对 `LEPTOS_OUTPUT_NAME` 和减少动态效果偏好。不能把某次旧开发资源与新导出资源错配，写成用户观察到某种特效问题的唯一根因；开发与静态浏览器流程须分别验收。无脚本、WASM 失败时正文与普通链接仍应可读；交互需要 hydration。完整测试矩阵见 [verification.md](verification.md)。

## 发布与恢复边界

workflow 的 `push` 仅匹配 `main`，每次推送都尝试部署该提交；`workflow_dispatch` 必须填写**本网站仓库**已推送的完整、小写 40 位十六进制 `source_sha`，不可填旧个人主页仓库的 SHA。workflow 仅在 `github.repository == 'MCB-SMART-BOY/mcb-smart-boy.github.io'` 执行 build：解析 SHA、以不保存 checkout 凭据的方式检出并校验实际 HEAD，准备 nightly 组件／target、执行 `cargo xtask setup` 和 `cargo xtask check`，然后上传同一 run 的 `run/pages` 并由 deploy job 发布。只有源码仓库现有 GitHub Actions Pages 设置、Pages 环境门禁和检查成功，发布链才可完成；推送未经本地验收的提交仍可能上线。不要以“CI 执行 check”替代本地独立质量门或线上验收。

工作流默认只授予 `contents: read`，仅 deploy job 获得 `pages: write` 和 `id-token: write`；各发布 Action 固定完整提交，`upload-pages-artifact` 使用固定 v4.0.0 提交，其内部 upload-artifact 也固定 v4.6.2 提交。checkout `persist-credentials: false`，未配置 PAT；发布使用 workflow 的 `GITHUB_TOKEN` 和 Pages 环境策略。并发组为 `github-pages`，`cancel-in-progress: false`：进行中的部署不因新推送取消；等待中的运行可能被更新推送替换，不能把每个 push 都误记为单独成功发布。

`run/provenance/source-sha.txt` 存实际检出 SHA，`tool-versions.txt` 存 `rustc -vV`、Cargo、cargo-leptos、wasm-bindgen、wasm-opt 实际版本；独立 `source-and-tool-provenance` artifact 保留期配置为 30 天，不混入网站。Pages artifact 从同 run 的 `run/pages` 上传；目前 `upload-pages-artifact` 上传时排除点文件，因此本地导出清单 `.mcb-static-export` 不进最终 Pages artifact，不能据此断言上线丢失了页面资源。provenance 能表明来源与工具版本，不等于字节可复现证明；滚动 nightly 使只凭 SHA 无法重建同样字节。

授权发布后要在 Pages 平台实际核对 HTTPS、深链刷新、无尾斜线请求、真实 HTTP 404、WASM MIME、资源完整性，记录**来源仓库、源码 SHA、workflow run**；本地纯静态服务或 SSR 成功均不能代替线上验证。发布失败停止后续发布并保留诊断，不虚构回滚基线。恢复前确认旧 run 经验证的 artifact 是否仍可取得；过期则按记录的**原来源仓库与 SHA**重建、验收后再另行授权发布。迁移前个人主页仓库的 SHA 不能传给当前同仓工作流；当前没有跨仓兼容回滚路径，既有线上回滚未演练，不能承诺立即恢复。涉及旧 workflow 重跑或远端设置变更须先确认仓库、输入、权限和实际影响。

项目本地验收先做 [行为矩阵](verification.md)，随后 `cargo xtask check`；本地还须独立执行 `"$HOME/.omp/agent/validate.sh"` 的适用质量门。分别记录结果，不把未运行、缺工具或跳过写成通过，也不要把仅针对网站 `Cargo.lock` 的审计扩展到共享安装工具。

## 历史来源与未消除风险

本节均为当时留存的迁移、验收和告警记录，**不是本轮重验或当前线上状态**；近期本地浏览器与资源错配记录另见 [verification.md](verification.md)。

### 2026-10-11 · 迁移归属与文件核对

旧网站源码基线为个人主页仓库 `MCB-SMART-BOY/MCB-SMART-BOY@867921977902e6cf4856bf17112c9fad72e186a5`；目标网站仓库当时发布配置基线为 `f8b608fa10c1f2d6e81d122215311def8626f234`。两个仓库各自保留 Git 历史，不强推或改写；旧仓库仍含网站源码，迁移并非隐私清理。本地整理当时不等于已提交、推送或上线，也不迁移构建缓存、导出产物、凭据和旧发布记录。

当时对 147 个网站源码、资源及构建文件核对过字节一致性与复制权限，随后以旧项目私有工具链完成 release 静态导出及浏览器 smoke；不是现行 `cargo xtask setup` 安装入口，更非本轮或现有 nightly 的验收。后来用户否决一轮界面重排、要求恢复旧仓库网站源码，仅保留工具链／包管理修改。恢复时核对 141 个源码、资源、内容及补丁文件，其中 139 个与上述旧源码 SHA 的 Git blob 相同；当时仅 `src/main.rs` 的构建提示和 `src/export.rs` 对 `Infallible` 的 nightly 兼容处理不同。后来单独修复 `vendor/leptos_router/src/flat_router.rs` 四处 release warning，未因此恢复被否决的界面重排；那一轮界面证据不能代表恢复后的页面。此后页面又继续变化，不能据 139/141 推断当前文件仍一致。

### 2026-10-11 · 恢复后的本地验证记录

恢复后使用 `rustc 1.101.0-nightly (6f5da406b 2026-10-10)`：`cargo xtask dev` 和 `cargo run --locked` 在浏览器接管成功，纯静态导出覆盖当时全部九个规范页面、两类未知路径 HTTP 404、历史往返、query/hash 刷新、中英界面、浅深主题与偏好恢复、侧栏、320px 移动菜单、真实 Clipboard 复制、无 JavaScript、WASM 失败／延迟和非法 cookie 回退。正常路径当时未观察到浏览器运行时错误；所用 Chromium 不支持 Web Share，**原生系统分享面板未验证**。四处 warning 修复后再次完成 release SSR/WASM 构建与静态导出，四条相应警告不再出现；纯静态浏览器再次覆盖当时九页、历史往返、深链刷新、移动阅读、偏好、真实复制及无脚本／WASM 后备。上述证据只针对恢复后的**当时版本**，不代表之后页面修改、本轮检查或 Pages 平台验收；无 warning 也不表示下述 RustSec 风险已消除。

### 2026-10-11 · 旧缓存清理

当时删除旧 `dev.sh` 隔离遗留 `run/{build,cargo,rustup,trivy,cache,config,data,tmp}` 与验收下载副本，不保留跨轮次缓存；旧 `run/build/release/` 不再是现行二进制路径。这是历史事实，**不是建议执行的清理命令**。后来的开发与导出资源错配、动效浏览器观察及其不能作为唯一根因的边界，见 [verification.md](verification.md)。

### 2026-10-11 · 迁移前线上基线

当时查得 [workflow run 37590809155](https://github.com/MCB-SMART-BOY/mcb-smart-boy.github.io/actions/runs/37590809155) 对应 Pages deployment `6905095541` 状态为 `success`。workflow 提交是网站仓库 `f8b608fa10c1f2d6e81d122215311def8626f234`；其 `source-commit` artifact 记录的实际网站源码却是旧个人主页仓库 `867921977902e6cf4856bf17112c9fad72e186a5`，两者不能互换。旧源码当时仍可读取，旧 Pages artifact 已不在可下载列表，**不能承诺直接恢复该产物**；如确需旧版本，先核对旧 workflow 是否可重跑、重建和重新验收旧源码，不得将个人主页 SHA 输入当前同仓工作流。线上回滚未演练。

### 2026-10-11 · 迁移后首次发布

[Workflow run 38140946750](https://github.com/MCB-SMART-BOY/mcb-smart-boy.github.io/actions/runs/38140946750) 手动输入网站仓库源码 `72a0c5ef8321f50375c03d9bfd7b7bd7bcc63490`，deployment 当时记录同一源码 SHA。provenance 记录 `rustc 1.101.0-nightly (6f5da406b 2026-10-10)`、`cargo 1.101.0-nightly (29c5daa1a 2026-10-07)`、cargo-leptos 0.3.11、wasm-bindgen 0.2.129、wasm-opt 116。那次 HTTPS 首页、`/writing/` 与文章深链返回 200；`/writing` 返回 301 到尾斜线，未知路径及旧 `/writing/books/` 返回真实 404。同 run Pages artifact 下载后与线上整棵树逐文件比对，字节一致（含哈希 CSS／JS／WASM、字体、图片与许可）。验收后本地删除 `run/`，线上产物不受影响；再查须确认平台 artifact 仍在保留期，或按记录的来源 SHA 重建重验。该 run **不代表当前线上版本**。

### 2026-10-11 · 构建工具依赖告警

三个工具各自发行包锁文件的额外审计仍有上游告警；cargo-leptos 0.3.11 正常安装的 lightningcss／parcel_sourcemap 链引入 `rkyv` 0.7.46，受 [RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235.html) 影响，修补版为不兼容的 0.8.17 及以上；单纯重解兼容依赖不能修复。未证明本站构建会触发漏洞，未加忽略项掩盖它；其他命中须分别核对 feature／target／调用路径。网站自身的审计即使通过，也不等于共享构建工具安全；本轮未重新审计或消除这项上游风险。

### 2026-10-14 · 网站锁文件更新

按现有 manifest 约束执行 `cargo update`，更新 `equivalent`、`manyhow`、`manyhow-macros`、`syn` 3、`tokio`、`zerocopy`、`zerocopy-derive` 共七项补丁版本；未新增包或依赖来源，固定的 Leptos／wasm-bindgen 版本与本地 router patch 不变。七项新版本的 checksum 均与 crates.io 官方索引一致，均未撤回；这只证明所下载包的完整性，不证明其源码或供应链无风险。未新增审计例外，也未重新安装或审计共享构建工具；上述 `rkyv` 风险及下述 `paste` 例外仍保留。

### 网站依赖例外与维护期限

[`deny.toml`](../deny.toml) 仅允许列出的许可证，对 Leptos 0.8 间接依赖 `paste` 的停止维护通告 `RUSTSEC-2024-0436` 按无直接可升修补版本记录例外。维护者应在 **2027-01-03 前**及升级框架后复核并清理不再需要的例外；`unused-ignored-advisory = "deny"`。2026-10-11 更新现有 manifest 约束后 `proc-macro-error2` 离开依赖图，对应旧例外已移除。例外不等于不存在风险，也不能代替工具发行包的独立审计。
