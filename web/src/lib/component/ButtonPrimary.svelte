<script lang="ts">
	import { goto } from '$app/navigation';

	export let href: string;
	export let className: string = '';
	export let external: boolean = false;

	function handleClick(event: MouseEvent) {
		if (external || href.startsWith('http://') || href.startsWith('https://')) {
			return;
		}

		event.preventDefault();
		goto(href);
	}
</script>

<a
	{href}
	class={`button-primary ${className}`}
	on:click={handleClick}
	target={external ? '_blank' : ''}
	rel={external ? 'noopener noreferrer' : ''}
>
	<slot></slot>
</a>

<style>
	.button-primary {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.5rem 1.2rem;
		color: var(--color-secondary);
		font-weight: 700;
		font-size: 1.2rem;
		text-decoration: none;
		text-align: center;
		border-radius: 0.5rem;
		transition: all 0.3s ease;
		border: 1px solid var(--color-secondary);
		cursor: pointer;
	}

	.button-primary:hover {
		background-color: var(--color-secondary);
		color: white;
	}
</style>
