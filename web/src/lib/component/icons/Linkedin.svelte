<script lang="ts">
	export let size = 24;
	export let color = 'currentColor';
	export let className = '';
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke={color}
	stroke-width="2.5"
	stroke-linecap="round"
	stroke-linejoin="round"
	class={`linkedin-icon ${className}`}
>
	<path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"></path>
	<rect x="2" y="9" width="4" height="12"></rect>
	<circle cx="4" cy="4" r="2"></circle>
</svg>
