<script lang="ts">
  import "../app.css";

  let { children } = $props();
</script>

<svelte:head>
  <title>dAstIll</title>
  <meta name="theme-color" content="#f8f1ec" />
  <meta name="application-name" content="dAstIll" />
  <meta name="apple-mobile-web-app-title" content="dAstIll" />
  <meta
    name="description"
    content="dAstIll - follow channels, process transcripts, evaluate summary quality, and manage your video distillation workspace."
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

<div class="flex min-h-screen flex-col">
  <div class="flex-1">
    {@render children()}
  </div>
</div>
