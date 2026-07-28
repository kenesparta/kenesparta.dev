use crate::app::components::PageMeta;
use leptos::prelude::*;
use leptos::{IntoView, component, view};

/// A role from the CV. Static data: edit an entry and redeploy, same as any
/// copy change (source of truth: cdn.kenesparta.dev/cv/ken_esparta_cv.pdf).
struct Role {
    title: &'static str,
    company: &'static str,
    period: &'static str,
    location: &'static str,
    summary: &'static str,
    highlights: &'static [&'static str],
}

const ROLES: &[Role] = &[
    Role {
        title: "Senior Software Engineer (Go, Rust)",
        company: "BairesDev LLC",
        period: "Oct 2022 — Present",
        location: "Remote, US",
        summary: "Manage and enhance partner system integrations based on their requirements.",
        highlights: &[
            "Engineered a Rust CLI utility to improve testing velocity and data verification \
             by automating AWS S3 and DynamoDB data workflows — locally downloading S3 \
             transaction data with automatic formatting and dynamically updating DynamoDB \
             credit check records.",
            "Engineered the complete foundational infrastructure, development lifecycle, and \
             tooling for a new partner integration, ensuring the secure, reliable display of \
             energy plans; established and deployed automated credit check verification as a \
             prerequisite to user plan enrollment.",
            "Defined project scope and requirements by conducting in-depth reviews of partner \
             API documentation, understanding the technical requirements, and coordinating \
             cross-organizational meetings.",
            "Executed the complete Software Development Life Cycle for the integration, \
             encompassing development, rigorous testing, and successful deployment to the \
             production environment.",
            "Engineered automated alerting and monitoring workflows to proactively detect \
             integration errors; drove incident resolution by generating detailed tickets \
             and executing immediate code fixes.",
            "Led coworkers through mentoring sessions on the enrollment business logic and \
             processes, accelerating team integration and boosting their proficiency with \
             core system functionality.",
        ],
    },
    Role {
        title: "Senior Software Engineer (Go)",
        company: "Quickplay",
        period: "Aug 2021 — Sep 2022",
        location: "Remote, Canada",
        summary: "Maintained and enhanced a microservices-based video-on-demand (VOD) platform.",
        highlights: &[
            "Optimized the codebase for the core VOD processing services (Audio, Video, \
             Subtitles, Media Assets, and DRM) to ensure compliance with multiple VOD \
             delivery standards and improve encoding quality consistency.",
            "Implemented comprehensive automated testing within the CI/CD framework, \
             enforcing quality control for microservice deployments and preventing issues \
             from reaching production.",
            "Applied continuous static code analysis and refactoring, improving average code \
             quality scores across the VOD platform microservices.",
            "Led new coworkers through end-to-end VOD encoding pipeline training and a \
             comprehensive quality assurance process, directly reducing team ramp-up time \
             and increasing overall productivity.",
        ],
    },
    Role {
        title: "Software Engineer",
        company: "Land Gorilla LLC",
        period: "Oct 2019 — Aug 2021",
        location: "Remote, US",
        summary: "Optimized a loan management system and on-site inspection APIs with enhanced security.",
        highlights: &[
            "Architected and deployed a private API for construction loan management, \
             integrating on-site inspection functionality and adhering to the OpenAPI \
             Specification (OAS) and architecture best practices.",
            "Significantly optimized API call efficiency for credential management, cutting \
             CPU usage by 50%.",
            "Enhanced API security by implementing an advanced encryption and authentication \
             layer for user session data.",
            "Developed a real-time system monitoring solution to automate and manage network \
             access controls for virtual machines.",
        ],
    },
    Role {
        title: "Software Engineer",
        company: "Freelancer",
        period: "Jan 2018 — Sep 2019",
        location: "Onsite, Peru",
        summary: "Built secure, efficient APIs for government health and agriculture offices.",
        highlights: &[
            "Architected a private REST API that significantly improved the efficiency of \
             documentary procedures for the Government Health Office, enhancing internal \
             workflow and information management.",
            "Created a private API for the Government Agriculture Office to register and \
             monitor farming production metrics, including seed-time, harvest data, and \
             livestock farming activities.",
            "Architected and configured the network infrastructure for both offices on a \
             three-tier architecture, implementing security-focused networking and \
             load-balancing to drastically improve application availability and ensure \
             database isolation.",
        ],
    },
    Role {
        title: "Software Engineer",
        company: "2Triangle",
        period: "Jan 2017 — Dec 2017",
        location: "Remote, Brazil",
        summary: "Developed a construction ERP to ensure item quantity and quality accountability.",
        highlights: &[
            "Designed and built the core functionality of an ERP web application that manages \
             end-to-end construction projects, specifically controlling the quantities and \
             quality standards for every build item.",
            "Optimized the financial workflow by improving and deploying the accountability \
             API to efficiently manage and process all project billing information.",
            "Improved data accessibility and inventory control for site personnel by \
             integrating the inventory API directly into existing company mobile \
             applications.",
        ],
    },
    Role {
        title: "Software Engineer",
        company: "Ware Digital",
        period: "Mar 2016 — Dec 2016",
        location: "Onsite, Brazil",
        summary: "Implemented a web application to streamline the management of patient orders.",
        highlights: &[
            "Architected a comprehensive web-based platform for managing patient orders and \
             clinical results, including configuring and securing the necessary cloud \
             infrastructure.",
        ],
    },
];

#[component]
pub fn Experience() -> impl IntoView {
    view! {
        <PageMeta
            title="Experience - Ken Esparta"
            description="Work history of Ken Esparta: senior software engineering roles building \
                         Go and Rust backend systems for energy, video-on-demand, and finance \
                         companies."
            path="/experience"
        />
        <div class="experience-container">
            <ol class="timeline">
                {ROLES
                    .iter()
                    .map(|role| view! { <TimelineEntry role=role/> })
                    .collect_view()}
            </ol>
        </div>
    }
}

#[component]
fn TimelineEntry(role: &'static Role) -> impl IntoView {
    view! {
        <li class="timeline-entry">
            <h2 class="timeline-title">{role.title}</h2>
            <div class="timeline-meta">
                <span class="timeline-company">{role.company}</span>
                <span>{role.period}</span>
                <span>{role.location}</span>
            </div>
            <p class="timeline-summary">{role.summary}</p>
            <ul class="timeline-highlights">
                {role
                    .highlights
                    .iter()
                    .map(|highlight| view! { <li>{*highlight}</li> })
                    .collect_view()}
            </ul>
        </li>
    }
}
