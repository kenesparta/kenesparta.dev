<svelte:head>
    <title>About</title>
    <meta name="description" content="About me and my work"/>
</svelte:head>

<div>
    <div class="about-content">
        <div class="about-text">
            <p>
                Hi there! I'm Ken, and I've been writing code professionally for over seven years now. My journey in
                software development has taken me through different industries and challenges, but there's one thing
                that has remained constant: my fascination with building systems that actually work well and can handle
                real-world pressure.
            </p>

            <p>
                I started my career back in 2016 at a small digital agency, learning the ropes with Python and Flask.
                Those early days taught me the fundamentals, but it was when I discovered Go that things really clicked
                for me. There's something about Go's simplicity and performance that just makes sense, especially when
                you're building APIs that need to be both fast and reliable.
            </p>

            <p>
                Over the years, I've worked on everything from government systems that help health and agriculture
                offices manage their operations, to video streaming platforms, and energy provider integrations. Each
                project taught me something new about designing systems that don't just work today, but can evolve and
                scale tomorrow.
            </p>

            <p>
                I have a particular interest in API security – following OWASP guidelines isn't just a checkbox for me,
                it's about building systems that people can trust. I've implemented everything from custom encryption
                layers to rate limiters that keep services stable under heavy load. It's the kind of work that might not
                be glamorous, but it's what keeps applications running smoothly when it matters most.
            </p>

            <p>
                These days, I'm working with serverless architectures on AWS, which has been an interesting shift from
                the more traditional approaches I used earlier in my career. There's something satisfying about building
                systems that automatically scale up and down based on demand, especially when you can see the direct
                impact on performance and costs.
            </p>

            <p>
                When I'm not debugging APIs or optimizing database queries, I'm usually exploring parallel and
                distributed computing concepts. It's one of those areas that seems endlessly complex but incredibly
                rewarding when you finally get it right. I also recently completed an MBA in Software Architecture,
                which helped me think more strategically about the systems I build and how they fit into the bigger
                picture.
            </p>

            <p>
                I believe good software should be like a well-written book – clear, purposeful, and easy to follow.
                Whether I'm working with a small startup or a large enterprise, my goal is always the same: build
                something that works, can be maintained, and actually solves the problem at hand.
            </p>


            <h2>Skills</h2>
            <ul class="skills-list">
                <li>JavaScript/TypeScript</li>
                <li>Svelte/SvelteKit</li>
                <li>HTML & CSS</li>
                <li>Node.js</li>
            </ul>
        </div>
    </div>
</div>

<style>
    .about-content {
        line-height: 1.6;
    }

    .about-text p {
        margin-bottom: 1.5rem;
        color: #555;
    }

    h2 {
        color: #333;
        margin-top: 2rem;
        margin-bottom: 1rem;
    }

    .skills-list {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 0.5rem;
        list-style: none;
        padding: 0;
    }

    .skills-list li {
        background: #f5f5f5;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        color: #333;
    }
</style>
