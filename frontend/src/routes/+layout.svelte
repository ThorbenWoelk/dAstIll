<script lang="ts">
  import "../app.css";
  import Footer from "$lib/components/Footer.svelte";
  import MobileViewportInset from "$lib/components/MobileViewportInset.svelte";
  import ServiceWorkerRegistration from "$lib/components/ServiceWorkerRegistration.svelte";

  let { children } = $props();
</script>

<svelte:head>
  <title>dAstIll</title>
  <meta name="application-name" content="dAstIll" />
  <meta name="apple-mobile-web-app-title" content="dAstIll" />
  <meta name="apple-mobile-web-app-capable" content="yes" />
  <meta name="apple-mobile-web-app-status-bar-style" content="default" />
  <meta name="mobile-web-app-capable" content="yes" />
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
  <MobileViewportInset />
  <ServiceWorkerRegistration />
  <div class="flex-1">
    {@render children()}
  </div>
  <Footer />
</div>
