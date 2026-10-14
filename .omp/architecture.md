# 系统架构与边界

本文描述现有代码的职责、数据流、SSR 与静态导出边界，以及本地路由补丁的来历；不是新增实现方案。
内容作者的字段和路径规则见 [content-format.md](content-format.md)，界面行为细节见 [design.md](design.md)，
操作与发布见 [maintenance.md](maintenance.md)，验收方法见 [verification.md](verification.md)。
以相应源码与实际构建结果为准，不把历史故障记录视作本轮重测。

## 模块分工

| 边界 | 事实源与职责 |
|---|---|
| 内容输入 | `content/posts/**` 的 `_index.md` 与文章 Markdown；`build.rs` 调用 `build_support/` 发现、解析、校验、排序和生成索引。 |
| 内容查询 | `src/content.rs` 包含生成的 `content_index.rs`，提供 `Directory`、`Post`、`ContentEntry`、`page_paths`、`find_directory`、`find_post`；`src/reading.rs` 解析阅读路由。 |
| 页面与导航 | `src/app.rs` 创建应用和 SSR 文档；`src/pages/` 组合首页、目录、文章与 404；`src/landing.rs`、`src/landing_motion.rs` 处理首页分区与过渡；`src/components/` 提供共享壳、导航、阅读树、面包屑及交互；`src/profile.rs` 是独立于文章索引的静态中英资料。 |
| Markdown | `src/markdown.rs` 在渲染时生成 HTML 和标题索引，并复用 `build_support/links.rs` 的相对内容链接解析；左树、面包屑和文章目录取真实索引／标题，不猜测文章所属栏目。 |
| 运行与导出 | `src/main.rs` 的默认命令只用于本地 Axum/Leptos SSR 服务，`export` 命令调用 `src/export.rs`；`src/server.rs` 注册索引路由，`src/export/assets.rs` 校验与复制资源，`src/export/output.rs` 管理归属与产物替换。 |
| 首屏与客户端 | `src/preferences.rs`、`src/locale.rs` 解析／保存 cookie；`src/app.rs::shell` 输出文档初值，`src/lib.rs::hydrate` 按 SSR 初值接管，`src/components/shell.rs::SiteShell` 再恢复浏览器偏好。 |
| 工程与发布 | `Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`.cargo/config.toml` 与 `xtask/` 约束构建；`.github/workflows/pages.yml` 是唯一 Pages 配置；`vendor/leptos_router` 由根 manifest 的 `[patch.crates-io]` 选用。 |

根 `README.md` 只承担面向人的项目简介；本目录保存 AI 工作约束与技术上下文。
GitHub 个人主页由独立的 `MCB-SMART-BOY/MCB-SMART-BOY` 仓库维护，不是本站源码。

## 从内容到浏览器

1. `build.rs` 递归扫描 `content/posts/`，调用 `build_support::generate_content` 写入
   `OUT_DIR/content_index.rs`；新增、移动、改标题或排序均须重建，正文不是运行时远程请求。
   YAML 解析仅用于 build/dev，完整消费文档并限制解析资源，不启用文件包含／属性展开；
   非法文件、链接或冲突使构建失败并带文件原因，不静默遗漏。
2. `build_support/mod.rs` 为目录和文章建立同一个树，检查普通目录与安全名称、无符号链接、
   规范 URL 冲突及相对 Markdown 内容链接目标。目录可以混放子目录与文章，也可以为空；
   同级目录／文章按 `weight` 再按路径排序，`POSTS` 按 `iso_date` 从新到旧、同日按路径排序。
   目录介绍只显示本目录正文。
3. `build_support/generate.rs` 生成 `Directory`、`Post`、`ContentEntry`，嵌入正文并用
   `LazyLock` 按需渲染；路径查询通过已排序的 `DIRECTORIES`、`POST_PATHS` 二分定位，
   `POSTS` 是完整的生成索引而非手写文章表。`src/content.rs::page_paths` 联合首页、
   全部目录、全部文章，为 SSR 注册和导出提供同一份页面列表；页面、最近文章预览、
   左树与面包屑也消费生成索引，不在别处维护页面登记表。
4. 本地服务的 `src/server.rs::build_app` 经官方 `LeptosRoutes` 按索引注册 GET/Async SSR 页面，
   并以框架的文件／错误 fallback 提供其他请求；这保留 executor、请求与 meta 上下文。
   `src/app.rs::App` 在 `<Router>`、`<SiteShell>` 内放置统一的 `SiteContent`，
   它根据路径展示首页或阅读内容；无效根路径及无效 `/writing/...` 最终使用相同的
   `NotFoundPage` 结构。首次 HTML 已含正文和真实链接，客户端不负责补出正文。
5. `cargo xtask export` 用独立 `run/site-release/` 构建 release SSR 与 WASM，只有构建成功
   才调用同次 cargo-leptos 构建、位于 Cargo metadata target directory 下相应 host/release
   的原生二进制 `export`；该二进制旁的 `hash.txt` 对应同次构建。开发构建使用 `run/site/`，
   不和 release 资源混用，也不另以不同配置重新编译导出二进制。`src/export.rs` 在内存里
   向 Axum router 请求每个 `page_paths()` 的 200 响应和 `/404.html` 的 404 响应，
   检查完整、非空且有 HTML 文档头、限定单页响应大小，再写入暂存输出；不绑定 HTTP 监听端口。
6. 静态发布物是目录内 `index.html`、独立 `404.html`、哈希命名 CSS/JS/WASM 和本地资源；
   HTML 引用应与同次构建资源一致。线上仅由静态托管提供文件，不运行 Axum，不运行 Node/Bun
   页面服务，不使用外部页面资产；导出包不包含开发 AutoReload／WebSocket 或服务器二进制。
   浏览器加载脚本和 WASM 后 hydration，接管链接导航、主题、语言、分享、侧栏和移动抽屉。
   无 JS 或 WASM 加载失败时，正文、真实页面链接和原生文章目录仍可用，
   这些动态操作不能承诺可用。

### 规范 URL 与错误页

- 首页为 `/`，阅读根为 `/writing/`；其他内容 URL 取决于目录／文件的真实位置，均以 `/`
  结束，绝不依赖独立 slug、permalink、别名或重定向表。例如
  `content/posts/demo-notes/systems/small-systems.md` 对应
  `/writing/demo-notes/systems/small-systems/`，导出为
  `writing/demo-notes/systems/small-systems/index.html`。
- `src/reading.rs::resolve_reading_route` 只接受 `/writing/` 及索引内路径；
  无尾斜线等非规范应用路径和已经删除的旧 `/writing/books/...`、旧章节 URL
  不会在应用内解析为现有文章。托管平台可能先对目录请求作重定向；
  重定向的实际状态码须上线后核验，不能拿本地静态预览的 301 充当 Pages 平台保证。
- `NotFoundPage` 在 SSR 有响应上下文时设置 HTTP 404，导出时据此要求
  `/404.html` 响应为 404 并保存该页面。`404.html` 不是首页副本或 JS 跳转；
  静态托管的未知路径实际响应仍取决于平台配置，不因本地 HTML 文件存在就保证线上状态。

## SSR 初值、颜色与 hydration

- 静态 HTML 无请求 cookie：固定中文 `zh-CN`、浅色、展开侧栏、`--sidebar-width: 248px`；
  `src/app.rs::shell` 把不可变接管初值放入 `data-ssr-ui` 和 `data-ssr-lang`。
  与之不同，本地 SSR 开发时 `src/app.rs::read_cookie_preferences` 读取请求 cookie
  以生成该请求的文档初值；不要把开发首屏当静态站点 cookie 个性化行为。
- `<head>` 中 `src/preferences.rs::THEME_BOOTSTRAP` 在 CSS 加载前检查 `mcb-ui`
  的名称、长度与格式，只提前设定合法的 `data-theme` 和 `theme-color`；
  不修改 `data-ui`、`lang` 或组件树，不做提前翻译。
  主题脚本的语法和数字边界需与 Rust `parse_preferences` 一致。
  英文 cookie 偏好在单份静态 HTML 上可能短暂看到中文 UI，这是设计取舍。
- `src/lib.rs::hydrate` 在挂载前从 `data-ssr-ui`／`data-ssr-lang` 解析 SSR 原始值
  并提供给 `<App/>`，不直接从 cookie 构造另一棵树。
  `SiteShell` 接管后通过带 `has_restored_browser_preferences` 守卫的 effect
  只读取浏览器 cookie 一次；同一批次恢复主题、语言、折叠状态和宽度，
  之后同步文档属性并标记 `data-hydrated`。恢复不是写 cookie，
  不能用默认值覆盖已有合法保存值，也不能把偏好加载当作用户点击以触发 Logo 飞入／水波。
- `mcb-ui` 的结构为 `light|dark:0|1:宽度`，长度至多 64 字节；
  宽度为合法 u16 时夹在 208–360px，非法、超长或读取失败回默认。
  语言只识别 `zh-CN` 和 `en`，非法值回中文；客户端只同步对应字段。
  偏好保存由用户交互触发。英语选择只翻译固定界面文本，
  仓库文章的标题／正文保持中文原文及 `lang="zh-CN"` 标识，不构成英文译本。
  UI 响应与交互详见 [design.md](design.md)。

### 已知构建耦合与历史故障

- `Cargo.toml` 的 `disable-erase-components = true` 保持服务端与 WASM 组件 hydration 标记一致；
  历史真实浏览器中移除此项导致顶栏按钮不可用并出现 `Unrecoverable hydration error`。
  升级 Leptos 或更改组件边界必须在浏览器回归首页、文章和交互；
  编译与 SSR 路由测试不足以证明接管成功。
- `src/lib.rs` 的 `#![recursion_limit = "256"]` 只提高编译期视图布局查询深度；
  合并三个完整首页分区后默认深度曾报 `queries overflow the depth limit`。
  不以运行时装箱或关闭 hydration 一致性设置掩盖此问题。
- `Cargo.toml` 中 `[profile.dev] debug = 1` 针对完整 Leptos 单态化导致链接器
  debug-section 超过 4 GiB 的历史问题。`.cargo/config.toml` 向普通 Cargo 构建提供
  `LEPTOS_OUTPUT_NAME`，否则 `cargo run` 曾请求不存在的 `_bg.wasm` 导致交互无法接管；
  这是应用资源命名约束，不是共享工具链隔离。上述都是已记录故障／约束，
  不代表本文重新触发或验证。

## 内容、链接及本地输入的安全边界

- 输入源是仓库自有 `content/posts/`，不是公开投稿接口。
  `build_support/metadata.rs` 校验 front matter；`build_support/links.rs`
  把相对 `.md`／`_index.md` 解析成规范内容 URL，并在构建时核对目标存在，
  拒绝越过内容根、歧义路径等；更细字段、格式和路径规则由
  [content-format.md](content-format.md) 维护。
- `src/markdown.rs` 对 raw `Html`／`InlineHtml` 事件按文本处理，
  避免其作为可执行 HTML 注入；链接和图片地址只允许 `https://`、`http://`、
  `mailto:`、非 `//` 开头的站内绝对路径及 `#` 锚点，其余替换为 `#`。
  渲染也会按同一解析逻辑改写有效相对内容链接；若解析出错则记录原因并退回 `#`，
  但预期构建时已经先拒绝无效内容链接。
  不要把这些有限检查描述为任意不可信 HTML／URL 的通用 sanitizer。
- `src/markdown/code_blocks.rs` 只在已经过消毒的代码块事件外插入固定结构，
  对未知语言 token 做 HTML 文本转义；`pre/code` 和正文仍由原 serializer 输出。
  `src/components/markdown.rs` 统一文章和目录介绍的 `inner_html` 边界，
  hydrate 后扫描当前正文，由 `Portal` 挂载控件；卸载时清理控件和 owner。
  异步完成先检查 owner 存活及代码节点连接状态，不把旧文章结果写到新文章。
  `src/clipboard.rs` 为代码复制和分享共用同步的 Clipboard Promise 调用，
  保留点击激活和原始 JS 错误；日志只记固定错误类别／允许名单错误名，
  不记代码、URL 或异常消息。只在显式点击时写入，不读剪贴板或执行代码。
- `public/` 只承载本地静态素材，颜色和布局由 `assets/site.css` 实现，
  本地字体许可保留在 `public/fonts/`；外部跳转、剪贴板和 Web Share
  等浏览器能力按组件各自的边界处理。
  个人资料与项目陈述属于 `src/profile.rs`／`src/components/profile.rs`，
  不是 Markdown 投稿或运行时 GitHub 数据源。具体界面和资产见 [design.md](design.md)。

## 导出完整性、归属与失败语义

- `src/export.rs::validate_source` 只接受 `run/site-release/pkg` 且 `hash_files` 已启用的配置；
  `assets::read_hashes` 从当前导出二进制同目录读取 `hash.txt`，
  强制 CSS、JS、WASM 哈希条目存在，检查合法／唯一哈希名及可选 WASM split
  对应条目，要求清单命名的资源真实存在。
  哈希文件名和清单做的是构建产物关联与缺失检测；代码没有逐字节重新计算 digest，
  不能声称密码学内容完整性证明。
- `assets::copy_assets` 接收约定的 pkg 扩展名和 public 文件类型，
  跳过 wasm-bindgen 的 `.ts` 类型声明；复制前拒绝空文件、非法 WASM 文件头、
  public 下保留的 `pkg` 路径或未预期资源。源目录须为真实目录；
  文件用 `symlink_metadata` 拒绝符号链接，Unix 上还拒绝多硬链接，目录递归也检查种类。
  `route_output` 拒绝非规范／不安全 URL 片段，
  `PreparedOutput` 拒绝不安全相对路径及文件／目录归属冲突。
- 在 `run/.pages-stage-*` 暂存目录中完成资源与全部页面后，
  `PreparedOutput::finish` 写 `.mcb-static-export` 归属清单。
  替换已有 `run/pages/` 前，`validate_previous` 要求它是真实目录，
  清单与实际文件、目录精确一致，不接受额外手工文件、缺失文件或链接；
  不要直接向导出目录添加自有文件。
- 准备阶段失败保留旧输出；若已有输出，先移动到 `run/.pages-backup-*/old`，
  再安装新目录；安装失败尝试将旧目录移回。
  若安装及恢复都失败，错误同时记录两个原因和保留的备份路径，需人工处理；
  安装成功后清理备份若失败仍会报告错误。
  不能承诺所有文件系统故障自动回滚，更不能在失败时改用上次导出的旧站作为本次构建结果。
- 文件／链接检查与清单面向可信的本地构建输入和非恶意操作失误；
  对拥有同等目录写权限、在检查与使用之间恶意并发替换文件的进程并非安全沙箱。
  本地 path 依赖源码不受 `Cargo.lock` registry checksum 校验，
  `--locked` 只锁定依赖图；供应链检查范围不得因为在 `vendor/` 就排除源码。
  审计例外、工具风险、构建和发布步骤归 [maintenance.md](maintenance.md)。

## vendored Leptos 路由补丁

- 根 `Cargo.toml` 的 `[patch.crates-io]` 将 `leptos_router = 0.8.16`
  定向到 `vendor/leptos_router`。已有出处记录：官方发行包与上游提交
  [`584c3a2d884b0e4dba9e3f822a9b6982cff072c7`](https://github.com/leptos-rs/leptos/tree/584c3a2d884b0e4dba9e3f822a9b6982cff072c7/router)；
  保留原 manifest、源码、README、构建脚本和 `.cargo_vcs_info.json`，
  MIT `LICENSE` 来自该提交的仓库根，局部 formatter 配置仅采用上游稳定选项。
  原始未修改 `src/location/history.rs` 的已记录 SHA-256：
  `d668d8477c2fa4324423b0df9c95dbc13fe08ea5bf2d31e4bde3b7ebd9c25e42`。
  这是补丁前文件的历史校验值，不是当前已补丁文件的哈希；
  升级时仍需与官方来源独立比对。源码修改限于以下两份文件，不碰其他发行包代码。
- `src/location/history.rs`：历史浏览器复现为 Back→Forward 后内部 `path_stack` 未同步，
  随后点击原目录出现正文改变而地址栏未变。补丁的 `update_from_history`
  在 `popstate` 时更新 URL、栈顶和方向估计；`commit_navigation`
  以浏览器实际当前完整 URL（含 query/hash）与目标比较决定是否 `pushState`，
  `replaceState` 不新增真实历史项且调整内部栈。
  浏览器提交错误保留原始 `JsValue` 并记录，不新增 panic；
  不在应用侧补写网址来掩盖分叉。
- `path_stack` 仅推测路由过渡动画的前后方向，
  无法区分重复 URL 在真实 History 中的游标；本站未启用该框架 transition，
  实际地址提交不依赖方向估计。不可据此声称内部栈完整复现浏览器历史。
- `src/flat_router.rs`：只在 `build`、`rebuild`、`hydrate`、`hydrate_async` 四处用
  `debug_assert!(child.is_none(), ...)` 替换 `#[cfg(debug_assertions)]`
  分支中的 `if`／`panic!`，消除 release 的四条 `unused variable: child` warning。
  调试构建仍拒绝嵌套路由，release 不执行该断言；不改变 `child`
  所有权／析构时点、不加 lint 忽略，也不为处理 warning 重写上游长函数。
- 升级框架前核对上游是否真的修复，再决定移除补丁；两份差异必须一起复核。
  实际浏览器回归主路径：`/writing/`→系统目录→Rust 目录→Rust 文章→Back→Forward→Rust 目录，
  核对地址、正文、面包屑、左树和再次 Back 的目标一致；
  还需覆盖两页往返、重复 URL、多步历史、query/hash、相同完整 URL 去重和 replace。
  Rust/SSR 测试不执行浏览器 History API，不能代替这组交互；
  验收与证据责任见 [verification.md](verification.md)。
  不得修改用户 Cargo registry 缓存冒充补丁。
