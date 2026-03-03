<script lang="ts">
	import "../app.css";

	let { children } = $props();
</script>

<svelte:head>
	<title>DASTILL v1.0</title>
	<meta name="theme-color" content="#f8f1ec" />
	<meta
		name="description"
		content="DASTILL v1.0 - follow channels, process transcripts, evaluate summary quality, and manage your video distillation workspace."
	/>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link
		rel="preconnect"
		href="https://fonts.gstatic.com"
		crossorigin="anonymous"
	/>
	<link
		rel="stylesheet"
		href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,100..900;1,9..144,100..900&family=Manrope:wght@200..800&display=swap"
	/>
</svelte:head>

{@render children()}
