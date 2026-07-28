use crate::app::components::{PageMeta, PersonJsonLd, SocialLinks, Tags};
use leptos::prelude::*;
use leptos::{IntoView, component, view};

/// Static CV data (source of truth: cdn.kenesparta.dev/cv/ken_esparta_cv.pdf).
struct SkillGroup {
    name: &'static str,
    items: &'static [&'static str],
}

struct Education {
    school: &'static str,
    detail: &'static str,
    period: &'static str,
}

const SKILLS: &[SkillGroup] = &[
    SkillGroup {
        name: "Programming Languages",
        items: &["Rust", "Golang", "Python", "JavaScript"],
    },
    SkillGroup {
        name: "Orchestration",
        items: &[
            "Docker",
            "GitHub Actions",
            "Terraform",
            "Kubernetes",
            "Apache Kafka",
        ],
    },
    SkillGroup {
        name: "Cloud Providers",
        items: &[
            "Amazon Web Services",
            "Google Cloud Platform",
            "Digital Ocean",
        ],
    },
    SkillGroup {
        name: "Databases",
        items: &["PostgreSQL", "MySQL", "MongoDB", "Redis"],
    },
    SkillGroup {
        name: "Languages",
        items: &[
            "English (Advanced)",
            "Spanish (Native)",
            "Portuguese (Fluent)",
        ],
    },
];

const EDUCATION: &[Education] = &[
    Education {
        school: "Let's get Rusty Bootcamp, US — Remote",
        detail: "Specialization in Software Development with the Rust programming language.",
        period: "Jul 2025 — Oct 2025",
    },
    Education {
        school: "Faculdade Full Cycle, Brazil — Remote",
        detail: "Specialization in Technical Leadership.",
        period: "Apr 2024 — Feb 2025",
    },
    Education {
        school: "Faculdade Brasília, Brazil — Remote",
        detail: "MBA in Software Architecture.",
        period: "Feb 2023 — Mar 2024",
    },
    Education {
        school: "Faculdade Brasília, Brazil — Remote",
        detail: "Specialization in Software Engineering.",
        period: "Sep 2023 — Mar 2024",
    },
    Education {
        school: "Federal University of Ceará, Brazil — Onsite",
        detail: "B.S. in Software Engineering.",
        period: "Jan 2013 — Dec 2016",
    },
];

#[component]
pub fn About() -> impl IntoView {
    view! {
        <PageMeta
            title="About - Ken Esparta"
            description="Senior software engineer with 8+ years of experience, specializing in \
                         Go and Rust backend microservices across energy, video-on-demand, and \
                         finance."
            path="/about"
        />
        <div class="about-container">
            <PersonJsonLd/>
            <p class="about-bio">
                "I'm a senior software engineer with 8+ years of experience, specializing in \
                 leading, architecting, and implementing highly efficient, highly secure \
                 backend microservices in Go and Rust. My expertise spans the full Software \
                 Development Life Cycle across diverse industries, including energy, \
                 video-on-demand, and finance. I focus on driving system efficiency — \
                 optimizing API call efficiency and architecting secure network \
                 infrastructure to improve application availability — and I'm committed to \
                 building robust, high-quality software with a keen attention to detail."
            </p>

            <SocialLinks/>

            <section class="about-section">
                <h2>"Skills"</h2>
                {SKILLS
                    .iter()
                    .map(|group| {
                        let items: Vec<String> =
                            group.items.iter().map(|item| (*item).to_string()).collect();
                        view! {
                            <div class="skill-group">
                                <span class="skill-group__name">{group.name}</span>
                                <Tags tags=items/>
                            </div>
                        }
                    })
                    .collect_view()}
            </section>

            <section class="about-section">
                <h2>"Education"</h2>
                <ul class="education-list">
                    {EDUCATION
                        .iter()
                        .map(|entry| {
                            view! {
                                <li class="education-entry">
                                    <div>
                                        <strong class="education-school">{entry.school}</strong>
                                        <p class="education-detail">{entry.detail}</p>
                                    </div>
                                    <span class="education-period">{entry.period}</span>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </section>
        </div>
    }
}
