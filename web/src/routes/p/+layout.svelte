<script lang="ts">
	import '../../global.css';
	import Navbar from '$lib/component/Navbar.svelte';
</script>

<Navbar />

<main>
	<slot />
</main>

<style>
	main {
		min-height: calc(100vh - 60px);
		padding: 2rem;
		max-width: 1000px;
		margin: 0 auto;
		font-size: 1.2rem;
	}
</style>
