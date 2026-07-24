use crate::app::components::{CodebergIcon, GithubIcon, Tags};
use leptos::prelude::*;
use leptos::{IntoView, component, view};

/// A showcased project. Static data: projects change rarely, so the list
/// lives here — edit an entry and redeploy, same as any copy change.
struct Project {
    name: &'static str,
    description: &'static str,
    tech: &'static [&'static str],
    github: Option<&'static str>,
    codeberg: Option<&'static str>,
}

const PROJECTS: &[Project] = &[
    Project {
        name: "kenesparta.dev",
        description: "This website: full-stack Rust with Leptos (SSR + hydration) and Axum, \
                      a Markdown-driven blog on PostgreSQL, deployed to AWS Lightsail behind \
                      CloudFront with Terraform.",
        tech: &["Rust", "Leptos", "Axum", "PostgreSQL", "Terraform", "AWS"],
        github: Some("https://github.com/kenesparta/kenesparta.dev"),
        codeberg: Some("https://codeberg.org/kenesparta/kenesparta.dev"),
    },
    Project {
        name: "fibonacci-wasm",
        description: "Fibonacci with Rust compiled to WebAssembly and packaged with Docker — \
                      built for a Docker Init Ayacucho talk.",
        tech: &["Rust", "WebAssembly", "Docker"],
        github: Some("https://github.com/kenesparta/fibonacci-wasm"),
        codeberg: None,
    },
    Project {
        name: "quiz-generator",
        description: "Quiz generation service written in Rust, with a TypeScript frontend \
                      in a separate repository.",
        tech: &["Rust", "TypeScript"],
        github: Some("https://github.com/kenesparta/quiz-generator"),
        codeberg: None,
    },
    Project {
        name: "education-platform",
        description: "Domain model for an education platform — Domain-Driven Design tactical \
                      patterns in Rust.",
        tech: &["Rust", "DDD"],
        github: Some("https://github.com/kenesparta/education-platform"),
        codeberg: None,
    },
    Project {
        name: "kcd-go-operator",
        description: "A Kubernetes operator written in Go, built for KCD Lima.",
        tech: &["Go", "Kubernetes"],
        github: Some("https://github.com/kenesparta/kcd-go-operator"),
        codeberg: None,
    },
    Project {
        name: "typst-resume",
        description: "My résumé as code: written in Typst and compiled/published via CI.",
        tech: &["Typst", "GitHub Actions"],
        github: Some("https://github.com/kenesparta/typst-resume"),
        codeberg: None,
    },
    Project {
        name: "zig-adventures",
        description: "Exercises and experiments while exploring the Zig language.",
        tech: &["Zig"],
        github: Some("https://github.com/kenesparta/zig-adventures"),
        codeberg: Some("https://codeberg.org/kenesparta/zig-adventures"),
    },
];

#[component]
pub fn Projects() -> impl IntoView {
    view! {
        <div class="projects-container">
            <div class="projects-grid">
                {PROJECTS
                    .iter()
                    .map(|project| view! { <ProjectCard project=project/> })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn ProjectCard(project: &'static Project) -> impl IntoView {
    let tech: Vec<String> = project.tech.iter().map(|t| (*t).to_string()).collect();

    view! {
        <article class="project-card">
            <h2 class="project-name">{project.name}</h2>
            <p class="project-description">{project.description}</p>
            <Tags tags=tech/>
            <div class="project-links">
                {project
                    .github
                    .map(|href| {
                        view! {
                            <a
                                href=href
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label=format!("{} on GitHub", project.name)
                            >
                                <GithubIcon size=22/>
                            </a>
                        }
                    })}
                {project
                    .codeberg
                    .map(|href| {
                        view! {
                            <a
                                href=href
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label=format!("{} on Codeberg", project.name)
                            >
                                <CodebergIcon size=22/>
                            </a>
                        }
                    })}
            </div>
        </article>
    }
}
