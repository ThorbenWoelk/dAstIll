<script lang="ts">
  import "../app.css";
  import AppBottomNav from "$lib/components/AppBottomNav.svelte";
  import Footer from "$lib/components/Footer.svelte";
  import GlobalKeyboardShortcuts from "$lib/components/GlobalKeyboardShortcuts.svelte";
  import MobileViewportInset from "$lib/components/MobileViewportInset.svelte";
  import ServiceWorkerRegistration from "$lib/components/ServiceWorkerRegistration.svelte";
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
</svelte:head>

<div class="flex h-screen flex-col overflow-hidden">
  <GlobalKeyboardShortcuts />
  <MobileViewportInset />
  <ServiceWorkerRegistration />
  <div class="min-h-0 flex-1">
    <slot />
  </div>
  <AppBottomNav />
  <Footer />
</div>
