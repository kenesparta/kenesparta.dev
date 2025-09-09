<svelte:head>
	<title>Projects</title>
	<meta name="description" content="My projects and work" />
</svelte:head>

<div>
	<p>Here are some of the projects I've worked on:</p>

	<div class="projects-grid">
		<div class="project-card">
			<h3>Project 1</h3>
			<p>
				Description of your first project. What technologies did you use? What problems did it
				solve?
			</p>
			<div class="project-links">
				<a href="" class="project-link">View Project</a>
				<a href="" class="project-link">GitHub</a>
			</div>
		</div>

		<div class="project-card">
			<h3>Project 2</h3>
			<p>
				Description of your second project. Highlight the key features and your role in the
				development.
			</p>
			<div class="project-links">
				<a href="" class="project-link">View Project</a>
				<a href="" class="project-link">GitHub</a>
			</div>
		</div>

		<div class="project-card">
			<h3>Project 3</h3>
			<p>
				Description of your third project. What challenges did you overcome? What did you learn?
			</p>
			<div class="project-links">
				<a href="" class="project-link">View Project</a>
				<a href="" class="project-link">GitHub</a>
			</div>
		</div>
	</div>
</div>

<style>

	h1 {
		color: #333;
		margin-bottom: 1rem;
	}

	.projects-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 2rem;
		margin-top: 2rem;
	}

	.project-card {
		background: #f9f9f9;
		padding: 1.5rem;
		border-radius: 8px;
		border: 1px solid #eee;
	}

	.project-card h3 {
		color: #333;
		margin-bottom: 1rem;
	}

	.project-card p {
		color: #555;
		line-height: 1.6;
		margin-bottom: 1.5rem;
	}

	.project-links {
		display: flex;
		gap: 1rem;
	}

	.project-link {
		color: #007acc;
		text-decoration: none;
		font-weight: 500;
		padding: 0.5rem 1rem;
		border: 1px solid #007acc;
		border-radius: 4px;
		transition: all 0.3s ease;
	}

	.project-link:hover {
		background: #007acc;
		color: white;
	}
</style>
