<svelte:head>
	<title>About</title>
	<meta name="description" content="About me and my work" />
</svelte:head>

<div class="page-container">
	<h1>About Me</h1>

	<div class="about-content">
		<div class="about-text">
			<p>
				Hello! I'm a developer passionate about creating amazing web experiences. This is where you
				can share your story, background, and what drives you.
			</p>

			<p>
				You can talk about your experience, skills, interests, and what you're currently working on
				or learning.
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
	.page-container {
		max-width: 1000px;
		margin: 0 auto;
	}

	h1 {
		color: #333;
		margin-bottom: 2rem;
	}

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
