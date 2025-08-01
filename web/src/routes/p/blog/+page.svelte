<script lang="ts">
</script>

<svelte:head>
	<title>Blog</title>
	<meta name="description" content="My blog posts and articles" />
</svelte:head>

<div class="page-container">
	<h1>Blog</h1>
	<p>Welcome to my blog! Here you'll find my latest posts and articles.</p>

	<!-- You can add blog post list here -->
	<div class="blog-posts">
		<article class="post-preview">
			<h2>Sample Blog Post</h2>
			<p class="post-date">January 1, 2024</p>
			<p>This is a sample blog post. You can replace this with your actual blog content.</p>
			<a href="/blog/sample-post" class="read-more">Read more →</a>
		</article>
	</div>
</div>

<style>
	.page-container {
		max-width: 1000px;
		margin: 0 auto;
	}

	h1 {
		color: #333;
		margin-bottom: 1rem;
	}

	.blog-posts {
		margin-top: 2rem;
	}

	.post-preview {
		border-bottom: 1px solid #eee;
		padding-bottom: 1.5rem;
		margin-bottom: 2rem;
	}

	.post-preview h2 {
		margin-bottom: 0.5rem;
		color: #333;
	}

	.post-date {
		color: #666;
		font-size: 0.9rem;
		margin-bottom: 1rem;
	}

	.read-more {
		color: #007acc;
		text-decoration: none;
		font-weight: 500;
	}

	.read-more:hover {
		text-decoration: underline;
	}
</style>
