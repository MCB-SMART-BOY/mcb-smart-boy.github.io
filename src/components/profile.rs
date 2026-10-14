use crate::{locale::Locale, profile::ProfileProject};
use leptos::prelude::*;

#[component]
pub(crate) fn ProjectCards(projects: &'static [ProfileProject]) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <ul class="project-grid">
            {projects.iter().map(|project| view! {
                <li class="project-card">
                    <h4><a href=project.url target="_blank" rel="noopener noreferrer">{project.name}<span aria-hidden="true">" ↗"</span></a></h4>
                    <p>{move || locale.get().select(project.summary_zh, project.summary_en)}</p>
                    <span class="project-stack">{project.stack}</span>
                </li>
            }).collect_view()}
        </ul>
    }
}

#[component]
pub(crate) fn CodeFlowCard() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div class="community-card">
            <img class="community-logo" src="/images/codeflow-logo.png" width="640" height="640" decoding="async" loading="lazy"
                alt=move || locale.get().select("CodeFlow·代码灵动社团 Logo", "CodeFlow·代码灵动 club logo")/>
            <div>
                <p>{move || locale.get().select(
                    "我创办了 CodeFlow·代码灵动社团，希望把对代码的好奇，变成一起动手、共同学习的日常。",
                    "I founded CodeFlow·代码灵动 to turn curiosity about code into hands-on learning together.",
                )}</p>
                <p>{move || locale.get().select(
                    "我希望围绕 Rust、Linux 与开源工具展开交流：从 Rust 的工具链、所有权和小工具开发，到 Linux 的命令行、开发环境与系统使用，在具体问题中理解代码与系统。",
                    "I hope to build discussions around Rust, Linux, and open-source tools: from Rust tooling, ownership, and small utilities to the Linux command line, development environments, and everyday system use, learning through concrete problems.",
                )}</p>
                <p class="community-note">{move || locale.get().select(
                    "也欢迎带着自己的问题、学习笔记或小项目交流；比起堆砌技术名词，我更在意把问题讲清楚、把尝试记录下来。",
                    "Questions, learning notes, and small projects are welcome. Clear explanations and documented experiments matter more to me than a list of buzzwords.",
                )}</p>
            </div>
        </div>
    }
}
