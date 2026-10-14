use crate::{
    components::profile::{CodeFlowCard, ProjectCards},
    landing::LandingSection,
    locale::Locale,
    profile::{PROFILE_PROJECTS, PROFILE_SUMMARY_EN, PROFILE_SUMMARY_ZH},
};
use leptos::prelude::*;

#[component]
pub(super) fn AboutSection() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <section id=LandingSection::About.id() class="landing-section landing-panel" tabindex="-1" aria-labelledby="about-title">
        <div class="landing-page landing-motion">
        <section class="page-intro about-intro" aria-labelledby="about-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" ABOUT / THE PERSON BEHIND THE LOG"</span>
            <div class="profile-intro-grid">
                <div>
                    <h2 id="about-title">{move || locale.get().select("你好，", "Hi, ")}<span class="accent">{move || locale.get().select("我是 MCB.", "I'm MCB.")}</span></h2>
                    <p>{move || locale.get().select(PROFILE_SUMMARY_ZH, PROFILE_SUMMARY_EN)}</p>
                    <a class="inline-link" href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">{move || locale.get().select("访问我的 GitHub ", "Visit my GitHub ")}<span aria-hidden="true">"↗"</span></a>
                </div>
                <img class="profile-logo" src="/images/mcb-logo.png" width="640" height="640" decoding="async"
                    alt=move || locale.get().select("MCB-SMART-BOY 个人 Logo", "MCB-SMART-BOY personal logo")/>
            </div>
            <nav class="profile-jump-links" aria-label=move || locale.get().select("跳转到资料章节", "Jump to profile sections")>
                <a href="/#codeflow" target="_self">{move || locale.get().select("社团", "Community")}</a>
            </nav>
            <div class="back-link"><a href="/#home" target="_self">{move || locale.get().select("↑ 返回顶部", "↑ Back to top")}</a></div>
        </section>
        </div>
        </section>
    }
}

#[component]
pub(super) fn ProfileSections() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <ProfileProjects locale/>
        <ProfileBackground locale/>
        <ProfileCommunity/>
    }
}

#[component]
fn ProfileBackground(locale: RwSignal<Locale>) -> impl IntoView {
    view! {
        <section id=LandingSection::Background.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="background-title">
            <div class="landing-motion">
                <h3 id="background-title">{move || locale.get().select("教育与技能", "Education & skills")}</h3>
                <p>{move || locale.get().select(
                    "2024.09 起，本科阶段学习软件工程，关注数据结构、操作系统、数据库与计算机网络。",
                    "Studying software engineering at undergraduate level since September 2024, with a focus on data structures, operating systems, databases, and computer networks.",
                )}</p>
                <ul class="project-grid">
                    <li class="project-card">
                        <h4>{move || locale.get().select("语言与后端", "Languages & backend")}</h4>
                        <p>{move || locale.get().select(
                            "Rust（Tokio、Actix-Web、SQLx、egui）、Python（FastAPI、PyTorch）、OCaml",
                            "Rust (Tokio, Actix-Web, SQLx, egui), Python (FastAPI, PyTorch), OCaml",
                        )}</p>
                    </li>
                    <li class="project-card">
                        <h4>{move || locale.get().select("系统与环境", "Systems & environment")}</h4>
                        <p>{move || locale.get().select(
                            "Linux / NixOS、Nix、Helix / Zed、可复现开发环境",
                            "Linux / NixOS, Nix, Helix / Zed, reproducible development environments",
                        )}</p>
                    </li>
                    <li class="project-card">
                        <h4>{move || locale.get().select("验证与 AI", "Verification & AI")}</h4>
                        <p>{move || locale.get().select(
                            "eBPF、控制流分析、Lean 4 / Isabelle、实时推理与计算机视觉",
                            "eBPF, control-flow analysis, Lean 4 / Isabelle, real-time inference and computer vision",
                        )}</p>
                    </li>
                </ul>
                <p>{move || locale.get().select(
                    "围绕开源工具链与跨平台分发持续实践。",
                    "Continuing to work on open-source tooling and cross-platform distribution.",
                )}</p>
            </div>
        </section>
    }
}

#[component]
fn ProfileProjects(locale: RwSignal<Locale>) -> impl IntoView {
    view! {
        <section id=LandingSection::Projects.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="projects-title">
            <div class="landing-motion">
            <h3 id="projects-title">{move || locale.get().select("项目与开源", "Projects & open source")}</h3>
            <p>{move || locale.get().select("n3v3（原 Neve）", "n3v3 (formerly Neve)")}</p>
            <ProjectCards projects=&PROFILE_PROJECTS/>
            </div>
        </section>
    }
}

#[component]
fn ProfileCommunity() -> impl IntoView {
    view! {
        <section id=LandingSection::Community.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="codeflow-title">
            <div class="landing-motion">
            <h3 id="codeflow-title">"CodeFlow·代码灵动"</h3>
            <CodeFlowCard/>
            </div>
        </section>
    }
}
