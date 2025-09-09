<script lang="ts">
	import { page } from '$app/state';

	const navItems = [
		{ href: '/p/about', label: 'About' },
		{ href: '/p/blog', label: 'Blog' },
		{ href: '/p/projects', label: 'Projects' }
	];
	let navTitle = $state('About');

	$effect(() => {
		const currentNavItem = navItems.find(item => item.href === page.url.pathname);
		navTitle = currentNavItem ? currentNavItem.label : 'About';
	});
</script>

<nav class="navbar">
	<div class="nav-container">

		<div class="nav-logo">
			<div class="logo-container">
				<div class="logo" role="img"></div>
			</div>
			<span class="nav-title">{navTitle}</span>
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
		background: rgba(255, 255, 255, 0.3);
		backdrop-filter: blur(7px);
		border-bottom: 3px solid var(--color-primary);
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
		display: flex;
		align-items: center;
	}

	.nav-menu {
		display: flex;
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.nav-title{
		padding-left: 0.5rem;
		color: var(--color-primary);
		font-weight: bold;
	}

	.logo-container {
		background-color: var(--color-primary);
		width: 40px;
		height: 40px;
		padding: 0.5rem;
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

	.nav-link {
		color: #666;
		text-decoration: none;
		font-weight: bolder;
		padding: 01rem;
		margin: 0;
		cursor: pointer;
	}

	.nav-link:hover,
	.nav-link.active {
		color: white;
		background: var(--color-primary);
	}

	@media (max-width: 768px) {
		.nav-menu {
			gap: 1rem;
		}
	}
</style>
