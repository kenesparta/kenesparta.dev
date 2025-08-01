<script lang="ts">
	import ButtonPrimary from '$lib/component/ButtonPrimary.svelte';
	import Github from '$lib/component/icons/Github.svelte';
	import Linkedin from '$lib/component/icons/Linkedin.svelte';

	const githubPage = 'https://github.com/kenesparta';
	const linkedinPage = 'https://www.linkedin.com/in/kenesparta/';
</script>

<div class="grid-container">
	<aside class="sidebar"></aside>

	<section class="content">
		<section class="content__section">
			<div class="logo-container">
				<div class="logo" role="img"></div>
			</div>
			<h1 class="title">Ken Esparta</h1>
			<h2 class="subtitle">Software Engineer</h2>
			<p class="slogan">Developing software that scales with your ambition.</p>
			<div class="content__contact">
				<ButtonPrimary href="">Learn more</ButtonPrimary>
				<ButtonPrimary href={githubPage} external={true}>
					<Github size={20} />
				</ButtonPrimary>
				<ButtonPrimary href={linkedinPage} external={true}>
					<Linkedin size={20} />
				</ButtonPrimary>
			</div>
		</section>
	</section>

	<aside class="sidebar"></aside>
</div>

<style>
	.grid-container {
		display: grid;
		grid-template-columns: 1fr 2fr 1fr;
		min-height: 100vh;
	}

	/* For tablets */
	@media (max-width: 1024px) {
		.grid-container {
			grid-template-columns: repeat(1, 1fr); /* 2 columns for tablets */
		}

		.slogan {
			font-size: 1rem;
		}
	}

	.sidebar {
		background-color: #fff;
		padding: 1rem;
	}

	.content {
		background-color: #fff;
		display: grid;
		padding: 1rem;
		place-items: center;
	}

	.content__section {
		display: grid;
		min-height: 100%;
		min-width: 100%;
		place-items: center;
		align-content: center;
	}

	.content__contact {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		column-gap: 1rem;
		margin-top: 1.8rem;
	}

	.logo-container {
		background-color: var(--color-accent);
		padding: 1rem;
		border-radius: 10%;
		width: 80px;
		height: 80px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.logo {
		background-image: url('/icon-wob.svg');
		background-repeat: no-repeat;
		background-position: center;
		background-size: contain;
		width: 100%;
		height: 100%;
	}

	.title {
		margin-top: 0.8rem;
		font-size: 2.6rem;
		font-weight: bolder;
		color: var(--color-primary);
		text-align: center;
	}

	.subtitle {
		margin-top: 0.5rem;
		font-size: 1.6rem;
		font-weight: bold;
		color: var(--color-accent);
		text-align: center;
	}

	.slogan {
		margin-top: 1.1rem;
		font-size: 1.5rem;
		font-style: oblique;
		background: linear-gradient(
			-30deg,
			var(--color-primary) 0%,
			var(--color-secondary) 50%,
			var(--color-accent) 100%
		);
		background-size: 200% auto;
		background-clip: text;
		-webkit-background-clip: text;
		color: transparent;
		text-align: center;
		animation: gradient-shift 3s ease infinite;
	}

	@keyframes gradient-shift {
		0% {
			background-position: 0 50%;
		}
		50% {
			background-position: 100% 50%;
		}
		100% {
			background-position: 0 50%;
		}
	}
</style>
