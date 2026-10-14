# MCB / LOG 项目上下文

本文件是 AI 协作入口与项目约束，不是面向访客的 README。源码、内容与 GitHub Pages 发布归 `MCB-SMART-BOY/mcb-smart-boy.github.io` 所有；本地目录为 `~/Projects/mcb-smart-boy.github.io`。

网站使用 Rust / Leptos 构建完整静态 HTML，浏览器通过 Rust/WASM 接管交互；Axum 只用于构建和本地开发，线上没有后端服务。首页是自然滚动的单页个人介绍，文章归档与阅读使用独立路由。没有登录、评论、在线编辑或 RSS。

## 文档路由与所有权

| 文档 | 服务对象与唯一详细职责 |
|---|---|
| [README.md](../README.md) | 面向访客与开发者：网站介绍、入口、快速开始、写作与导出；不放 AI 指令、本机 Harness 要求或内部验收流水账 |
| [design.md](design.md) | 页面组织、视觉语言、响应式、导航、分享、动效、焦点及无脚本后备的设计契约 |
| [architecture.md](architecture.md) | 内容索引、SSR/hydration、偏好恢复、路由、导出安全与 vendored router 补丁 |
| [content-format.md](content-format.md) | 内容文件、元数据、目录／日期排序、网址与 Markdown 链接规范 |
| [maintenance.md](maintenance.md) | 标准工具链、开发／导出／发布操作、产物职责、迁移与发布历史、已知依赖风险 |
| [verification.md](verification.md) | 行为验收矩阵、项目检查、独立质量门、已执行证据及未验证边界 |
| 本文件 | 任务路由、跨模块约束、资料与授权边界；不再复制上述全文 |

- **代码与实际配置是当前行为的事实源。** 按任务读相关详细文档，变更时同步其负责文件和受影响入口，不维护互相矛盾的第二份契约。
- 历史记录保留日期和适用版本，不能覆盖为当前事实。文档中的源码 SHA、workflow run、页数或测试数都是对应记录的证据，不是未来运行的固定输入。
- 网站文章及 `_index.md` 是内容，不是工程说明；不因整理文档而改写作者正文。字体许可证、归属声明和第三方 `vendor/leptos_router/README.md` 保持上游材料原样，不挪进 AI 目录或改写成本站介绍。
- 项目级设计、实现、维护及 AI 上下文统一放在 `.omp/`；根 README 是面向读者的入口。不要另建散落于仓库根的内部 `.md`。

## 当前产品约束

- 首页正文与 `LandingSection::ALL` 同序：首屏、关注领域、项目与开源、教育与技能、CodeFlow、最近记录、关于。关于是最后一个内容分区，后面的 GitHub 收尾入口与 footer 保留。
- 已删除独立经历、荣誉与交流分区及其导航，不保留空锚点、别名或重定向；旧 `/focus`、`/about` 不再是页面。具体真实锚点归 [design.md](design.md)。
- 首页只预览日期倒序、同日路径升序的最新三篇；完整内容索引和归档不裁剪。仓库包含原有示例及后来新增文章，不把全部内容说成“三篇示例成果”，不重写用户新增文章。
- 教育与技能复用项目卡片的布局语言；两种主题共用布局，只改变色彩。保留暖纸网格、紫黑配色、本地字体、Logo、卡片悬停、首页几何装饰、分区进出场和 Logo 飞入／水波，不借内容整理重新设计界面。
- 分享有明确用途与可见状态，取消不自动复制，失败不伪装成功；原生系统能力和受控替身的证据分开记录。详细状态、可访问性与旧请求失效契约见 [design.md](design.md)。
- 普通滚动只更新当前位置与动效，不改 URL、历史或焦点；真实锚点保持原生语义。无 JavaScript、WASM 失败或减少动态偏好时不能隐藏正文。
- 当前界面沿用户选定的旧网站视觉基线演进，不把恢复前被否决的重排当成现行设计；源码来源和历史核对记录归 [maintenance.md](maintenance.md)。

## 代码与职责速查

下列路径是定位入口，不是需要手工维护的文章或路由清单。

| 修改目标 | 事实源 |
|---|---|
| 文章与目录 | `content/posts/**`、各级 `_index.md` 和 front matter |
| 内容发现、校验、排序与生成 | `build.rs`、`build_support/`；通用查询在 `src/content.rs` |
| 应用、SSR、阅读路由与错误页 | `src/app.rs`、`src/server.rs`、`src/main.rs`、`src/reading.rs`、`src/pages/mod.rs` |
| 首页各节与最近文章 | `src/pages/home.rs`、`about.rs`、`focus.rs`、`post_list.rs` |
| 首页导航状态与滚动过渡 | `src/landing.rs`、`src/landing_motion.rs`；样式在 `assets/site.css` |
| 公开简介、项目与社团 | `src/profile.rs`、`src/components/profile.rs`、`src/pages/about.rs` |
| 外壳、侧栏、顶栏与分享 | `src/components/shell.rs`、`sidebar.rs`、`topbar.rs`、`share.rs` |
| 阅读树与面板切换 | `src/components/reading_navigation.rs`、`reading_directory.rs` |
| 文章目录、高亮与响应式焦点 | `src/components/article_outline.rs`、`current_directory.rs`、`directory_focus.rs` |
| 主题、侧栏 cookie、语言与接管 | `src/preferences.rs`、`src/locale.rs`、`src/lib.rs`、`src/components/shell.rs` |
| Markdown HTML、代码块与链接边界 | `src/markdown.rs`、`src/markdown/code_blocks.rs`、`build_support/links.rs`；客户端复制见 `src/components/markdown/`，共享 Clipboard 写入见 `src/clipboard.rs` |
| 静态导出、资源和归属替换 | `src/export.rs`、`src/export/assets.rs`、`src/export/output.rs` |
| 样式、字体、品牌与图标 | `assets/site.css`、`public/fonts/NOTICE.txt`、`public/images/`、`public/favicon.svg`、`src/components/icons.rs` |
| 构建、工具链、审计与发布 | `rust-toolchain.toml`、`Cargo.toml`、`Cargo.lock`、`.cargo/config.toml`、`xtask/`、`deny.toml`、`.github/workflows/pages.yml` |
| 路由依赖补丁 | `vendor/leptos_router/src/location/history.rs`、`flat_router.rs`；升级要求见 [architecture.md](architecture.md) |

## 资料、安全与授权

- 个人资料只采用用户已批准公开的事实，只公开 `MCB` / `MCB-SMART-BOY` 与 GitHub；不补真实姓名、联系方式、证件照、完整简历、合作单位或内部业务数据，不补造学校、性能数字或证明成功率。
- CodeFlow·代码灵动为作者创办的社团；Rust、Linux、开源工具是希望交流的方向，不暗示已举办活动、课程或产生成果。B站入口仅使用用户提供的 [社团主页](https://space.bilibili.com/3493283751791185)，不新增视频嵌入、二维码或账号调查。外站曾返回验证码，链接配置正确不等于账号内容已核验。
- 本仓库不维护第二份 GitHub 个人主页。该主页的简介与 `profile-*.svg` 位于独立 `~/Projects/MCB-SMART-BOY` 仓库；涉及它时先读该仓库上下文，不自动同步本站资料。
- 网站源码公开也会公开其中资料。只改 README 不构成源码隐私隔离；历史迁移不等于清除旧 Git 历史。
- 未经明确授权不暂存、提交、推送、创建仓库、发布、回滚或更改远端设置。**推送 `main` 会触发发布流程**；授权与恢复前提见 [maintenance.md](maintenance.md)。
- 不把 `run/`、本地 `.env`、缓存或凭据纳入源码。也不通过修改用户 Cargo registry 缓存代替本地版本化补丁。
- 不把输入检查、静态导出清单或依赖审计通过说成无条件安全保证；已知工具风险、审计例外及期限保留在 [maintenance.md](maintenance.md)。

## 实施与验收

1. 先确认用户可感知的目标及对应事实源。只改任务相关内容，不借修复交互扩大重构或重排页面。
2. 内容只能从文件扫描生成目录、文章、页面与导航索引；不要手写登记表、固定页数或按标签猜测归属。
3. 组件或资源变化必须保持 SSR 与 WASM 相配。`disable-erase-components`、编译深度、资源命名、cookie 初值和导出失败语义是有历史故障依据的约束，按 [architecture.md](architecture.md) 维护。
4. 日常只使用标准 Cargo/Rustup 与已准备的工具。不要隐式运行会替换共享工具的 `setup`，不覆盖用户工具链／缓存环境变量；开发资源与静态导出分别构建，详见 [maintenance.md](maintenance.md)。
5. UI 变更须验证实际画面、交互、焦点和动效，不以代码仍在、编译或 SSR 测试代替；纯文档变更核对事实、链接、命令与历史范围，不伪称重做了页面验收。
6. 完成任务匹配的证明后，从项目根运行 `cargo xtask check`，再独立运行 `"$HOME/.omp/agent/validate.sh"`；详细清单与证据分工见 [verification.md](verification.md)。`--quick`、跳过或缺工具不能作为通过。
7. 提交、推送和发布仍各有授权边界。交付报告只写实际完成的检查，未验证的原生系统分享、读屏、Pages 或回滚能力分别说明。
