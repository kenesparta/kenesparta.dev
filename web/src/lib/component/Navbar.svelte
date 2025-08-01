<script lang="ts">
	import { page } from '$app/state';

	const navItems = [
		{ href: '/p/about', label: 'About' },
		{ href: '/p/blog', label: 'Blog' },
		{ href: '/p/projects', label: 'Projects' }
	];
</script>

<nav class="navbar">
	<div class="nav-container">

		<div class="nav-logo">
			<div class="logo-container">
				<div class="logo" role="img"></div>
			</div>
			Your Logo
		</div>

		<ul class="nav-menu">
			{#each navItems as item}
				<li class="nav-item">
					<a href={item.href} class="nav-link" class:active={page.url.pathname === item.href}>
						{item.label}
					</a>
				</li>
			{/each}
		</ul>
	</div>
</nav>

<style>
	.navbar {
		background: rgba(255, 255, 255, 0.5);
		backdrop-filter: blur(7px);

		border-bottom: 3px solid var(--color-secondary);
		position: sticky;
		top: 0;
		padding: 0;
		margin: 0;
		z-index: 100;
	}

	.nav-container {
		max-width: 1000px;
		margin: 0 auto;
		padding: 0 0;
		display: flex;
		justify-content: space-between;
		align-items: center;
		height: 50px;
	}

	.nav-logo {
		font-size: 1.5rem;
		font-weight: bold;
		color: #333;
		text-decoration: none;
	}

	.nav-menu {
		display: flex;
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.nav-link {
		color: #666;
		text-decoration: none;
		font-weight: bolder;
		padding: 01rem;
		margin: 0;
		/*transition: color 0.3s ease;*/
		cursor: pointer;
	}

	.nav-link:hover,
	.nav-link.active {
		color: white;
		background: var(--color-secondary);
	}

	@media (max-width: 768px) {
		.nav-menu {
			gap: 1rem;
		}
	}

	.logo-container {
		background-color: var(--color-accent);
		padding: .4rem;
		border-radius: 10%;
		width: 20px;
		height: 20px;
	}

	.logo {
		background-image: url('/icon-wob.svg');
		background-repeat: no-repeat;
		background-position: center;
		background-size: contain;
		width: 100%;
		height: 100%;
	}
</style>
