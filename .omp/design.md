# 网站视觉与交互设计

本文记录现有网站的界面选择及维护边界，不是另起一版设计稿。设计结构以 [`HomePage`](../src/pages/home.rs)、[`SiteShell`](../src/components/shell.rs)、[`site.css`](../assets/site.css) 和相应组件为准；内容字段与排序来源见[内容规范](content-format.md)，站点构建、路由数据和偏好恢复的实现见[架构](architecture.md)，检查场景见[验证矩阵](verification.md)，操作命令见[维护说明](maintenance.md)。下文的尺寸和分区均指当前源码，而非历史截图或旧运行记录。

## 设计语言与资产

网站是一份可自然滚动的个人首页，以及独立的文章目录／文章路由。界面强调「保持好奇，持续构建」、项目与问题空间、清楚的阅读层级；不把关注方向写成已完成成果，不以动画妨碍阅读。首屏大字、几何轨道、稀疏刻度和紧凑文章列表由本地 CSS 绘制，不依赖第三方动画服务。

### 配色、字体与 Logo

[`assets/site.css`](../assets/site.css) 的 `:root` 和 `html[data-theme="dark"]` 共用 20px 细网格、圆角、间距、几何装饰与布局，仅交换色彩变量：浅色暖纸底、深灰文字、靛蓝强调；夜间紫黑底、浅紫文字与亮紫强调。`--signal` / `--on-signal` 用于强调按钮及文字，`--focus` 和 `:focus-visible` 提供可见焦点。不要为另一语言或主题复制页面结构。

`@font-face` 自托管 Maple Mono NF、JetBrainsMono Nerd Font Mono、思源黑体、思源宋体 Regular/Bold；中文正文使用本地字族，阅读正文 `.prose` 用 `--serif`，代码用 `--mono`。微软雅黑、仿宋只是访客系统字体回退，不是本站分发资产。字体是各自作者的作品，不属于站点原创品牌；包来源、版本、SHA、许可及 Nerd Fonts 字形归属见 [`public/fonts/NOTICE.txt`](../public/fonts/NOTICE.txt)。更换或裁剪较大的 CJK OTF 须核对字形覆盖与许可。

个人 Logo 与 CodeFlow Logo 分别在 [`public/images/mcb-logo.png`](../public/images/mcb-logo.png)、[`public/images/codeflow-logo.png`](../public/images/codeflow-logo.png)；组件以原有 640×640 图显示，CSS 白底 `object-fit: contain` 避免裁切。个人 Logo 归本站个人展示使用，CodeFlow Logo 只用于社团展示，不与字体、外站图标或独立 GitHub 个人主页资源混同；保持原有图形，不推断或转让第三方品牌权属。使用处是 [`sidebar.rs`](../src/components/sidebar.rs)、[`about.rs`](../src/pages/about.rs) 和 [`profile.rs`](../src/components/profile.rs)；人物与项目事实源另见 [`src/profile.rs`](../src/profile.rs)。

### 容器与断点

- `.site-content` 以 `width: min(calc(100% - 96px), 1260px)` 居中且声明 `container: page-content / inline-size`；workspace 容器宽度 ≤1000px 时减至 64px 边距，移动视口 ≤767px 时为 `min(calc(100% - 36px), 600px)`。文章页 `.article-page` 限宽 850px，正文 `.article-body` 限宽 72ch；长文保留行距、边界和标题层级。
- `@container workspace (max-width: 1000px)` 把双列 Hero 和三列关注卡变成单列；`@container page-content (max-width: 900px)` 把项目卡变两列，≤600px 变单列；这些边界含临界值。不要拿浏览器外窗宽度代替实际容器宽度。移动视口 ≤767px 时改为抽屉；顶栏另受 workspace ≥960px 的容器断点控制。
- Hero 标题按当前文案语言及 `.hero-copy` 容器宽度缩放，英文仍保持两行，不维护第二份英文首页。正文 `.prose` 与首页网格共享字体、配色和响应式选择，但不把文章塞进项目卡网格。

## 首页：七节结构与内容边界

[`src/pages/home.rs::HomePage`](../src/pages/home.rs) 的 `.landing-section` 外层顺序固定：

1. `#home` 首页（Hero 与 Ticker）。
2. `#focus` 关注领域。
3. `#open-source` 项目与开源。
4. `#background` 教育与技能。
5. `#codeflow` CodeFlow 社团。
6. `#notes` 最近记录。
7. `#about` 关于。

[`src/landing.rs::LandingSection::ALL`](../src/landing.rs) 是导航／滚动使用的同一七节顺序，`href()` 提供 `/#...`，`id()` 派生目标 ID；不额外造一份排序表。About 是最后一个**内容分区**，后面仍有 [`Closing`](../src/pages/home.rs) 的 GitHub 行动入口与 [`SiteShell`](../src/components/shell.rs) 的站点 footer；两者不是新分区。

旧工程／研究经历和荣誉／交流不是当前主页分区；`/#experience`、`/#recognition` 无匹配内容锚点、导航项或占位别名，旧 `/focus`、`/about` 也不是单独页面。个人资料片段链接使用普通 `<a target="_self">` 绕过 Router 点击拦截，由浏览器原生 hash / 跨页文档导航负责定位。

### 首页与关注领域

`#home` 双列 Hero：左侧中英两行标题、简介与「阅读文章」「了解我」，右侧 CSS 轨道与字形是纯装饰并有 `aria-hidden`；Ticker 是方向信号而非成果清单。[`FocusSection`](../src/pages/focus.rs) 的三张卡分别是系统与开发工具、形式化验证、AI 与工程实践；前两卡链接对应项目，第三卡不伪造无关项目外链。

### 项目、教育与社团

[`ProfileProjects`](../src/pages/about.rs) 与 `ProfileBackground` 共用 `.project-grid` / `.project-card`：默认三列，`page-content` 宽 ≤900px 两列、≤600px 单列。教育三项是**信息卡**（语言与后端、系统与环境、验证与 AI），不是项目链接；本科软件工程学习起始时间、课程及实践段落仍在卡片前后，不补造学校、进度条或成果。

每卡沿用项目卡的 24px 内边距、16px 网格间隔、面板／边框／圆角与 23px 标题、14px 正文；不要恢复旧侧标签分栏，也不要伪造卡片可点击状态。

`#codeflow` 使用 [`CodeFlowCard`](../src/components/profile.rs) 的 Logo + 三段中英文文案：社团由作者创办，**希望**围绕 Rust 工具链、所有权与小工具、Linux 命令行／开发环境／系统使用及开源工具交流，也欢迎具体问题、笔记和小项目。「希望」是方向而非已举行活动、课程或成果的声明；不添加成员规模或视频嵌入。`.community-card` 在正文容器 ≤600px 时由图文双列变单列，Logo 保持完整可见。

### 最近记录与完整归档

[`PostList`](../src/pages/post_list.rs) 只对完整 `POSTS` 索引 `iter().take(RECENT_POST_LIMIT)`，当前预览限制为 3；顺序来自构建生成索引的文章日期倒序、同日路径升序，不在界面复制排序逻辑。内容不足三篇时显示实际数量；「全部文章」始终进入 `/writing/` 的完整归档，不能为首页预览裁剪全库或删除更早文章。这与删除经历、荣誉首页分区无关。

中文原文的标题和摘要用 `lang="zh-CN"`；英文界面不代表文章已翻译。正文容器 ≤600px 时 `.post-row` 改紧凑栅格并隐藏首页预览中的日期列，文章本身的日期仍存在。内容字段与构建索引归[内容规范](content-format.md)、[架构](architecture.md)。

## 顶栏与移动入口

[`Topbar`](../src/components/topbar.rs) 左侧是当前位置面包屑，右侧依次语言、主题、GitHub、CodeFlow B站、分享；不再有居中的用户名占位。workspace 容器宽 ≥960px 时页头 72px、面包屑与操作同一行；在较窄桌面 workspace <960px 时为 100px、面包屑独占第二行；移动视口 ≤767px 时为 96px，第二行同样给面包屑。`--topbar-height` 同时用于分区／文章标题 `scroll-margin-top`，改页头须一起检查锚点可见位置。

面包屑长项在可横向滚动的空间里截断当前项文本，不挤掉操作。各顶栏按钮至少 44×44px，不为分享图标再塞窄屏常驻文本。语言按钮显示 `EN` / `中`（指向要切换到的语言），只翻译固定 UI；主题按钮沿用同一布局和视觉变量。保存语言失败时保留当前切换并显示可读的保存错误提示；cookie／SSR 偏好恢复机制归[架构](architecture.md)。

外链使用明确可访问名称、`target="_blank" rel="noopener noreferrer"`：GitHub 指向 `https://github.com/MCB-SMART-BOY`；B站指向用户提供的 `https://space.bilibili.com/3493283751791185`。链接用途不是对外站账号内容的核验。

### 移动抽屉

视口 ≤767px 隐藏桌面侧栏，用顶栏菜单键打开原生 [`dialog#mobile-drawer`](../src/components/shell.rs)；品牌尺寸相同，菜单和关闭键均有名称。点击首页内容锚点会关闭抽屉，并在目标仍在根文档时聚焦该分区；文章链接关闭并聚焦正文。阅读树目录链接、阅读根入口与主导航／阅读区切换保留抽屉开启；关闭或 Escape 后返回菜单键。无 hydration 时 `.mobile-fallback-nav` 提供真实主导航链接，不呈现空菜单。

## 分享：用途、结果与失效

### 能力选择和点击时序

[`ShareButton` / `ShareStatus` / `SharePanel`](../src/components/share.rs) 共用一个操作状态。按钮 `aria-label` 是「分享当前页面」，`title` 和常驻 `#share-description` 说明「支持时打开系统分享，否则复制链接」；hydration 前与处理中均禁用。真实点击先清旧反馈和面板，读取当前页面**完整 URL** 与标题，在失去用户激活之前直接调用 `navigator.share`；**仅 Web Share 不存在**时调用 Clipboard `writeText`。已存在的 Web Share 若拒绝、权限失败或能力异常，不默默改走复制，更不假报成功。

### 可见状态与手动回退

| 情况 | 用户看到的结果 |
|---|---|
| 处理中 | 「正在分享当前页面…」，无关闭键，重复点击不重新开始。 |
| Web Share Promise 成功 | 「已调用系统分享」；只确认调用，不断言接收方收到。 |
| Clipboard 成功 | 「链接已复制，可以粘贴分享」。 |
| Web Share `AbortError` | 「已取消分享」；不是失败，不自动复制。 |
| 无法取得链接 | 「无法获取当前页面链接」；不呈现假的手动 URL。 |
| 分享／剪贴板失败且已取得链接 | `.share-panel` 区分失败原因，展示真实 URL 的只读输入框，聚焦并全选供手动复制；不伪装成功。 |

完成的 `.share-status` 不自动消失，持续到关闭、再次点击或导航变化；「关闭提示」按钮、非忙碌且有反馈时的 Escape 清状态并把焦点还给分享按钮，成功本身不抢焦点。手动面板关闭或 Escape 也返回按钮。提示与面板共享距按钮下缘 8px、右对齐、最多 310px 且不超视口的面板语法（[`site.css`](../assets/site.css)）。

### 焦点、播报与过期请求

唯一常驻 `.sr-only` 播报节点 `role="status" aria-live="polite" aria-atomic="true"` 从 SSR 起就在可访问性树内；可见卡的重复文字 `aria-hidden="true"`，空文案时卡片由 `hidden` 收起。手动面板出现时 live region 为空，面板另用 `role="alert"` 报错。可访问属性是设计实现，不代替实际读屏验证；场景见[验证矩阵](verification.md)。

分享请求同时比对 generation、完整 URL、pathname/search/hash 与组件存活状态；导航清空旧状态并令旧 Promise 完成后的结果失效，hash 变化也不使上一页反馈复现。实现见 [`share.rs::watch_navigation`](../src/components/share.rs)。未加载 JS/WASM 时按钮不可用，但正文及真实链接仍可阅读。

## 侧栏 Logo、两面板和阅读目录

### 侧栏宽度与 Logo 飞入

[`Sidebar`](../src/components/sidebar.rs) 为常驻桌面栏，默认宽 248px，折叠 64px；展开可调范围 208–360px，最大值还受视口减 448px 正文保留空间限制（常量在 [`src/preferences.rs`](../src/preferences.rs)）。品牌容器高 68px，个人 Logo、折叠键／展开 Logo 各 44×44px；展开时折叠键在 Logo 右侧，折叠后只保留 Logo 展开入口。

侧栏项目至少 48px 行高、名称可换行；折叠的七个锚点图标与归档入口在可独立滚动区域，不让滚轮穿透正文。拖动分隔条使用 pointer capture，取消可回滚；键盘左右每次 16px、Home/End 到边界、Escape 取消当前拖动，并以 `aria-valuenow` 等暴露宽度。

折叠时先在真实按钮还可测量时取得飞入起点，立即移除展开态按钮／交互，下一可取消帧测量**折叠后** Logo 中心，再让 `.brand-flight` 飞入（CSS 180ms）；抵达后依 CSS 延时出现两圈同心水波（每圈 560ms，第二圈错开 90ms）。装饰层 `aria-hidden` 且不接收指针；快速展开取消待执行帧／装饰。首次以已保存折叠偏好打开、悬停、聚焦不会触发飞入；系统减少动态效果时直接切换状态。目标与焦点生命周期在 [`SidebarBrand`](../src/components/sidebar.rs)，时序和几何变量在 [`site.css`](../assets/site.css)。

### 纵向双面板与滚轮

主导航用 [`LandingSection::iter_navigation()`](../src/landing.rs) 给展开栏、折叠栏及移动抽屉提供七节；高亮与面包屑共用活动状态。内部 `/#...` 链接走原生导航，修饰键不重写。

[`ReadingNavigation`](../src/components/reading_navigation.rs) 的主导航与阅读导航是**两个纵向面板**，上下滑动切换（CSS 260ms），不随目录层级横向翻页；隐藏面板 `inert` + `aria-hidden`，短视口中面板自身滚动、标题固定在上方，归档入口位于主导航面板底部。主导航滚轮先滚完整长列表，到底且手势累计达阈值后进入 `/writing/`；仅在阅读**根** `/writing/` 且左树滚动到顶时向上才返回首页。目录／文章深页只滚动、不凭滚轮跳出阅读。

单个手势翻页后短暂锁定以抑制惯性回跳，切面后若原焦点在面板内，必要时转移到目标标题。`PageDown` / `ArrowDown` 等键盘切换仅在非交互元素上生效，不抢夺目录链接、按钮的原生键盘行为；减少动效偏好时面板无过渡。

### 左侧内容树

[`ReadingTree`](../src/components/reading_directory.rs) 用原生 `nav > ul > li` 呈现真实目录和文章。目录**整行是真实链接**，箭头只是装饰，不是第二个按钮；普通点击进入目录并展开，再点击当前目录只切换展开而不新增历史，修饰键保留浏览器原义。

根下默认展开排序后的首个目录，深链自动展开所有真实祖先，其余手动展开状态保留至组件销毁；标题可换行，行高至少 44px，缩进最多按三级计算。当前页用 `aria-current="page"`，目录链接通过 desktop/mobile 独立 ID 的 `aria-controls`、`aria-expanded` 对应子列表。SSR 时子列表全展开，hydration 后才显示箭头并收起非活动分支；无脚本仍可访问所有真实链接。

### 右侧与正文中的本文目录

文章有 Markdown 小标题时，[`ArticleOutline`](../src/components/article_outline.rs) / [`CurrentDirectory`](../src/components/current_directory.rs) 使用同一标题状态。`.site-workspace` 宽 ≥1100px 显示正文右侧 224px sticky「本文目录」，不足时显示正文原生 `details.article-outline-compact`；目录根页、普通目录、无小标题文章均不留空右栏，也不插入第二棵内容树。1100px CSS 断点须与 [`directory_focus.rs::CURRENT_DIRECTORY_MIN_WORKSPACE_PX`](../src/components/directory_focus.rs) 一致，折叠／拖动左栏会改变 workspace 宽度。

右栏外缘距离窗口右侧固定 16px，由 `--directory-edge-gap` 控制，不再随 1260px 居中容器向内收缩。右栏宽 224px、列间隔至少 32px；文章主列保持原来的 1004px 最大宽度（1260 − 224 − 32），文章本身仍最多 850px，避免超宽窗口放大正文和标题。无右栏页面的居中布局不变。

跨断点时可取消帧在**当下实际可见**的右栏标题或 compact summary 之间迁移焦点，不展开 details、不滚动正文；用户改换焦点取消旧请求，路由切换的正文焦点请求优先。

两个目录中的链接都指向真实 Markdown 标题，不重复文章标题。标题节点按当前文章取用；滚动、resize、hashchange、popstate 合并采样，只更新两处目录链接 `aria-current="location"`，不改历史、URL 或焦点；滚动到底选末标题，允许 1px 坐标取整容差。换页清理旧标题监听与待执行帧。右侧目录只从 Markdown 标题生成，树和面包屑只按内容目录索引组织；数据来源见[架构](architecture.md)与[内容规范](content-format.md)。

### 代码块标识与复制

代码框保留原来的字体、背景、边框、内边距和横向滚动，不新增工具栏行。语言小标签与复制图标嵌在右上角边框处；控件不占正文高度，也不覆盖第一行代码。未标记语言的代码仍显示 `Text`；标记规则见[内容规范](content-format.md)。

复制图标接管后才出现，使用原生按钮与可访问名称，支持鼠标、Enter 和 Space。只复制对应 `code` 的文本，保留显示代码的缩进、空行和末尾换行，不包含标签、图标或提示。进行中禁用重复点击；成功改为勾号，悬停／聚焦时显示“已复制”浮层。失败提示保持可见直到重试，明确说明可手动选择复制；反馈绝对定位，不撑高代码框。固定 UI 随语言切换，代码不翻译。

每块仅一个常驻 `sr-only` live region，视觉反馈 `aria-hidden`，无重复播报源。无 JavaScript 或 WASM 失败时仍有语言和完整代码，可原生选择复制，但不显示无效的复制按钮。真实读屏效果须另外实测，不能由这些属性推断。

## 首页滚动过渡与静态后备

### 稳定锚点与视觉进出

`.landing-section` 的外层 ID、尺寸、`scroll-margin-top: calc(var(--topbar-height) + 16px)` 保持稳定，不让视觉 transform 改变浏览器锚点几何。Hero、Focus 和 About 的 `.landing-panel` 至少占一屏，其余资料分区自然高度；正文不拦截滚轮、不强制吸附或内部滚动。仅直接子 `.landing-motion` 随滚动逐节过渡：从下方上移渐显；离场缩小、上移并淡出；反向滚动恢复，长章节中段保持清晰。没有 CSS scroll timeline 或 GSAP。

[`landing.rs::tracking`](../src/landing.rs) 缓存七节外层几何目标与内层动效节点；scroll / resize / hashchange / popstate、ResizeObserver 与系统动态偏好变化排入**可取消的单帧采样**。每帧先读所有外层几何，再由 [`landing_motion.rs::SectionMotion`](../src/landing_motion.rs) 写内部 `--landing-travel`；短节接近顶部才缩退，长节尾段接近离场才缩退，避免中途阅读时淡化。

### 活动节、焦点与生命周期

活动节采用顶栏底缘下 16px 加 1px 容差的判定线，与 CSS 锚点留白相配；滚动到底选 About。普通滚动只更新活动节／面包屑与动效，不写 URL、历史或焦点；SSR 初始 Home，hydrate 后按真实布局修正，避免初始 hash 导致标记分叉。

默认与 SSR 内容可见，只有 `html[data-hydrated="true"]` 且 `prefers-reduced-motion: no-preference` 才启用 CSS opacity/transform；动效偏好无法读取或设置减少动效时保持静态。分区自身或内部存在键盘可见焦点时恢复不透明且不位移，防止焦点淡出。无 JavaScript 时仍能读正文、点击真实链接、使用原生段落目录；跳过正文链接在获得焦点时显现。

离开根路由或组件销毁时取消待执行帧、断开 ResizeObserver，并移除窗口／动效偏好监听；文章标题和侧栏焦点也各自清理旧帧与监听。检查行为分支与减少动效场景见[验证矩阵](verification.md)。

## 设计参考，不是素材来源

- [Matklad](https://matklad.github.io/) 的单栏文章、[Anthony Fu](https://antfu.me/posts) 的列表密度、[Dan Luu](https://danluu.com/) 的长文归档、[Tarik Karahodžić](https://www.tarikkarahodzic.dev/projects/personal-site) 的留白，以及 [Stefan Vitasović 案例](https://tympanus.net/codrops/2025/03/05/case-study-stefan-vitasovic-portfolio-2025/) 的几何构图，是版式观察；本站用自身 CSS、文案和资源实现，不复制第三方素材。
- 浅色网格／正文底色参考本账号 [resume-template 的 `assets/styles/resume.css`](https://github.com/MCB-SMART-BOY/resume-template)，借鉴暖纸、20px 网格、靛蓝与深灰，不复制 A4 导出用的整张背景图。[Codrops 分区滚动过渡](https://tympanus.net/codrops/2024/01/31/on-scroll-animation-ideas-for-sticky-sections/) 提供缩退／接替的构图参考，不引入 GSAP。
- 阅读导航参考 [Docusaurus shared sidebar](https://docusaurus.io/docs/sidebar) 的稳定内容树／独立页内目录、[mdBook SUMMARY](https://rust-lang.github.io/mdBook/format/summary.html) 的层级，以及 [W3C APG Disclosure Navigation](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/examples/disclosure-navigation/) 的普通导航语义；本站采用目录整行链接，不照搬范例的单独展开按钮。内容 `_index.md` 约定来源见[内容规范](content-format.md)。
