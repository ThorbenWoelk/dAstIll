<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>DASTILL v1.0</title>
	<link rel="icon" href={favicon} />
	<meta name="theme-color" content="#f8f1ec" />
	<meta
		name="description"
		content="DASTILL v1.0 - follow channels, process transcripts, evaluate summary quality, and manage your video distillation workspace."
	/>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		rel="stylesheet"
		href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,500;9..144,700&family=Manrope:wght@400;500;600&display=swap"
	/>
</svelte:head>

{@render children()}
