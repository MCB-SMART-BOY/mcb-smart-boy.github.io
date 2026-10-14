---
title: "这里是 Rust 学习的万恶之源"
date: "2026-10-12"
summary: "Rust 还没开始写，工具的名字先来了一堆。从安装 Rustup 到 cargo run，把第一个项目跑起来。"
category: "Rust From Scratch"
reading_minutes: 8
weight: 10
---

好，开始学 Rust。

才怪，在使用编程语言之前，首先要考虑的是环境问题。

如果你第一次接触编程，下面会有几个陌生的名字。我会边用边解释，命令也可以跟着敲。已经装过 Rust 的话，直接跳到创建项目那一节就行。

我们这次要跑的是 `Hello, world!`。学过别的语言的同学大概又见到它了，国服来了还是这几句（狗头🐶）。

## 工具链们~~wcnm~~这群小可爱

首先要提到的其实是 `rustc`，这是 Rust 的编译器。我们写的 `.rs` 文件交给它编译，生成的程序才能运行。

不过，平时写项目很少需要直接调用它。新建项目、编译、运行，以及下载别人写好的库，通常都交给 `cargo`，这东西是我们后面最常用的工具。你敲 `cargo run`，Cargo 就会去调用编译器，编译成功后再运行程序。听不懂没关系，后面有实践部分，现在当个乐子也行。

`rustup` 这东西比 `rustc` 见的更多，也是我们接下来所有操作的起点。它管的是 Rust 工具链本身。安装 Rust、换版本、补组件，都用它。通过 Rustup 安装一套工具链时，`rustc` 和 `cargo` 会一起装上。

还有一个名字很长的 `rust-analyzer`，编辑器的补全和跳转要用到它，后面再装。

拿隔壁大蟒蛇 *Python* 来说，我更愿意用 `uv` 或 `pixi` 把项目环境分开，而不是像某些吃百家饭没个统一标准的语言一样，话虽如此，作为一个 *Python* 黑子，我还是要叫两句的 —— 闹麻了一个脚本语言真把自己当全能整？（笑）Rust 这边倒不用先挑一套项目管理工具，跟着官方的 `rustup` 和 `cargo` 来就行。

~~某些静态类型语言（说你呢C艹）是不是应该反思一下自己的问题。~~

## 安装 Rust（相关工具链）

先打开终端。Windows 可以从开始菜单打开 PowerShell；macOS 可以搜索 terminal ；Linux 用户就不用我多说了。

接下来的命令都在终端里输入，一行一行执行。输入完一行，按回车，再看它的输出。

安装步骤按你的系统选一节看，需要联网下载。

### Windows

打开 [Rust 官方安装页](https://rust-lang.org/tools/install/)，下载适合你电脑架构的 `rustup-init.exe`，运行后按提示安装。普通 Intel / AMD 的 64 位电脑通常选 x64，ARM 设备则选 ARM64。

默认安装的 MSVC 工具链还需要链接器和 Windows SDK。安装器提示缺少 Visual Studio 构建工具时，让它继续安装；手动配置可以看 [Rustup 的 MSVC 安装说明](https://rust-lang.github.io/rustup/installation/windows-msvc.html)，选择“使用 C++ 的桌面开发”工作负载。

怎么学 Rust 还要勾 C++？这里用的是它提供的构建工具，暂时不用去学 C++。

注意名字：**Visual Studio 和 Visual Studio Code 是两个软件**。给 VS Code 装扩展，补不上这里缺的链接器。

装好后重新打开 PowerShell，再往下检查工具链。

> 已经在用 WSL 的话，就在 WSL 的 Linux 终端里按下一节安装。Windows 和 WSL 各有自己的开发环境，要在实际写代码的那一边安装。没用过 WSL 的话，也不用为这一篇额外装它。（当然你装了就是半只脚加入 Linux 了，记得安装 arch 加入我们神教）

### macOS 和 Linux

> **NixOS 要单独看。** 请按 [NixOS Wiki 的 Rust 说明](https://wiki.nixos.org/wiki/Rust) 准备环境。如果已经用 Nix 管理 `rustc` 和 `cargo`，就跳过本文的 Rustup 安装和工具链管理命令，沿用 Nix 的配置。（不过我想都用上 `nix` 了应该不需要我来提醒吧？）

macOS 和常见 Linux 发行版可以按 [Rust 官方安装页](https://rust-lang.org/tools/install/) 的方式安装：

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

这条命令会从 `sh.rustup.rs` 下载脚本，然后交给 `sh` 执行。确认地址和官方安装页一致再运行，后面按安装器提示操作，不要给整条命令加 `sudo`。

骗你的，都用上类 Unix 了，还不会用包管理器那真的是废了。Mac用户请自行搜索 `homebrew` 使用方法，Linux 用户自己想（包管理器不会用那真是白瞎了 除非你的是Linux from scratch —— 那这入门课程也不适合你）。

本机的链接工具也要有：

- **macOS**：缺少命令行开发工具时，运行下面的命令，按弹窗提示安装。

  ```sh
  xcode-select --install
  ```

- **Linux**：按发行版说明准备 GCC 或 Clang 等本机构建工具。例如 Ubuntu 常用 `build-essential`；其他发行版的包名不一定相同。

安装完成后，重新打开一个终端，让命令搜索路径生效，然后输入：

```sh
rustup --version
```

能打印版本号就继续。提示找不到命令的话，可以先看文末的排错部分。

## 看看装好了没有

下面的 Rustup 命令，Windows、macOS 和使用 Rustup 的 Linux 用户都适用。

这份教程用 stable，也就是稳定版。nightly 上有还在试验的功能，入门暂时用不上。官方安装器通常已经装好并选中了 stable；如果你安装时选了别的版本，想改用 stable，可以执行下面两条命令。

> 第二条会修改当前用户的默认工具链，其他没有单独指定工具链的项目也会受影响。

> 不想改默认值的话，只执行第一条，随后在本文的 `cargo`、`rustc` 后加 `+stable`，比如 `cargo +stable run`。

```sh
# 使用 rustup 安装 stable 版本的 rust 相关工具链
rustup toolchain install stable

# 选择 stable 版本工具链作为默认的版本
rustup default stable
```

第一条负责安装，第二条设置默认使用哪一套工具链。懒得思考的可以直接复制粘贴到终端食用～

接着查看当前目录实际用的版本（虽然这些操作没什么作用，就是让你知道你做的没问题）：

```sh
rustup show active-toolchain
rustc --version
cargo --version
```

默认值设成 stable、目录也没有其他配置时，第一条应显示 stable，后两条会打印版本号。具体数字比本文新没关系。

如果你选择保留原默认值，第一条仍显示原来的工具链也正常；用 `rustc +stable --version` 和 `cargo +stable --version` 检查就好。已经改了默认值却还是别的版本？检查当前目录或父目录里的 `rust-toolchain.toml`、目录覆盖设置，以及 `RUSTUP_TOOLCHAIN` 环境变量。它们可以覆盖默认值，详细顺序在 [Rustup 文档](https://rust-lang.github.io/rustup/overrides.html) 里。

`rustc --version` 也可以写成 `rustc -V`。大写的 `V`。嫌命令长的同学可以少敲几个字母了。

## 编辑器里的 rust-analyzer

现在在终端里已经能找到 Rust 了。回到编辑器，代码补全、类型提示和跳转到定义这些功能，由 `rust-analyzer` 提供。

用 VS Code 的话，在扩展里搜索 `rust-analyzer`，选发布者为 `rust-lang` 的那个，按 [官方说明](https://rust-analyzer.github.io/book/vs_code.html) 安装。扩展通常自带分析器，装完就不用再执行下面这条安装分析器的命令了。

其他编辑器如果需要外部的分析器，可以用 Rustup 安装：

```sh
rustup component add rust-analyzer
```

分析标准库需要 `rust-src`。rust-analyzer 通常会尝试自动安装，也可以自己补上：

```sh
rustup component add rust-src
```

其他编辑器还需要按各自的 [接入说明](https://rust-analyzer.github.io/book/installation.html) 配置语言服务。稍后打开项目时，**打开包含 `Cargo.toml` 的整个文件夹**，这样分析器才能找到项目；只打开一个 `main.rs` 文件，补全可能不正常。

> 编辑器就用你顺手的。我先不在这里展开比较了。我是不会和你推荐什么我觉得比 `vscode` 速度更快个人体验更流畅舒适的 `Zed` 编辑器和什么终端编辑器比如什么 `helix` 和什么 `neovim` (`?!nixvim!?`) 的。

## 创建项目

选一个放练习的文件夹，例如 `~/Projects/`，Windows 上也可以用自己建的练习目录。先不要把新项目套进另一个 Rust 项目里。

在选好的文件夹里打开终端，输入：

```sh
# Rust From Scratch for short ~
cargo new rfs
cd rfs
```

`rfs` 是 *Rust From Scratch* 的缩写，也是这里的项目名。`cargo new` 会创建这个文件夹，`cd rfs` 则让终端进入它。

如果提示 `rfs` 已经存在，先看看里面有什么，换个项目名或换个空位置继续。不要直接把原来的文件夹删了。

> 前面用 Rustup、又保留了原默认工具链的同学，进入 `rfs` 后再执行 `rustup override set stable`，然后打开编辑器。这项设置只对这个目录和子目录生效，不改全局默认值。命令里的 `+stable` 只管那次命令，编辑器不会跟着切换，所以这里给项目也选一下。

用编辑器打开 `rfs` 文件夹或者使用 `tree` 的指令（如果没有请思考是不是自己系统的问题），会看到 Cargo 生成的文件。

Windows 自带的 `tree` 默认只列目录，要把文件也显示出来，得用 `tree /f`。我们先看这两个：

```tree
rfs/
├── Cargo.toml
└── src/
    └── main.rs
```

`Cargo.toml` 记录项目名称、版本和依赖。另一个 `src/main.rs` 放程序入口，打开它，里面已经有代码了：

```rust
fn main() {
    println!("Hello, world!");
}
```

所以第一份代码其实是 Cargo 替我们写的。

`main` 是这个程序开始执行的地方，里面的 `println!` 会输出一行文字。那个感叹号有它的用途，讲宏时再细说。

回到终端，确认还在 `rfs` 文件夹里，运行：

```sh
cargo run
```

Cargo 会先编译，再运行。`Compiling` 表示正在编译项目，`Finished` 表示构建完成，随后 `Running` 会显示准备运行的程序路径。这几行是 Cargo 的提示，最后才是程序自己的输出：

```text
Hello, world!
```

这行是程序的输出，不用再把它当命令敲一遍。

不改代码，再执行一次 `cargo run`，通常就看不到 `Compiling` 了：Cargo 会复用上次的构建结果，直接运行程序。等你修改并保存代码，下次运行时它才需要重新编译。

你可以把 `main.rs` 里的 `Hello, world!` 改成 `你好，Rust！`，保存，再运行一次 `cargo run`。这回终端里的字会跟着变。记得保留文字两边的英文双引号。

第一次构建后，项目里还会多出 `Cargo.lock`，构建产物通常放在 `target/` 文件夹里。`Cargo.lock` 记录依赖解析出的具体版本，由 Cargo 管理，现在不用手动改它。

`target/` 里有生成的可执行文件和编译过程中的缓存。后面写代码还是去改 `src/` 里的文件，别钻进 `target/` 找源码；删掉它后，Cargo 下次构建会重新生成，只是得重新编译。

程序跑通就先到这里，`rfs` 文件夹留着，后面还会用。

## 报错了再来看这里

如果前面某一步没成功，先留着执行的命令和完整报错。下面是几个比较容易对上的情况。

### **找不到 `rustup`、`rustc` 或 `cargo`**

先重新打开终端试一次。还是找不到，就按 [官方安装说明](https://doc.rust-lang.org/book/ch01-01-installation.html#troubleshooting) 检查 `PATH`。它是一组目录，终端会去这些目录里找你输入的命令。

Rustup 的命令通常放在 macOS / Linux 的 `~/.cargo/bin`，Windows 则是 `%USERPROFILE%\.cargo\bin`。如果安装时改过位置，就看你实际装到哪里了。

### **能查版本，编译时却找不到 `cc` 或 `link.exe`**

回头看安装部分的本机构建工具。macOS 要检查命令行开发工具，Linux 检查 GCC 或 Clang，Windows 检查 MSVC 构建工具和 SDK。这时反复重装 Rust 通常帮不上忙。

### **找不到 `Cargo.toml`**

看看终端现在在哪个文件夹。前面的 `cargo new rfs` 只创建项目，不会自动进入它，还要执行 `cd rfs`。进入包含 `Cargo.toml` 的目录，再运行 `cargo run`。

### **终端能运行，编辑器没有补全**

先确认编辑器打开的是整个 `rfs` 文件夹，再检查 rust-analyzer 扩展或语言服务有没有启用。程序已经能在终端运行的话，先不用动编译器。

### **下载失败或超时**

看报错里哪个地址下载失败了，检查网络后再重试。不要为了下载成功就关掉证书校验，也不用急着卸载。

还有对不上的错误，可以带着系统名称、刚敲的命令和完整报错去查、去问。尤其别只截最后一句 `could not compile`，有用的信息往往在它前面。
