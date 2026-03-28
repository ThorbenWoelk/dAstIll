<script lang="ts">
  import "../app.css";
  import { afterNavigate, goto } from "$app/navigation";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import type { AuthContext } from "$lib/auth";
  import AppBottomNav from "$lib/components/AppBottomNav.svelte";
  import { cleanupLegacyClientStorage } from "$lib/auth-storage";
  import { authState } from "$lib/auth-state.svelte";
  import {
    authRequiredNotice,
    dismissAuthRequiredNotice,
    presentAuthRequiredNoticeIfNeeded,
  } from "$lib/auth-required-notice";
  import SignInRequiredModal from "$lib/components/SignInRequiredModal.svelte";
  import GlobalKeyboardShortcuts from "$lib/components/GlobalKeyboardShortcuts.svelte";
  import MobileViewportInset from "$lib/components/MobileViewportInset.svelte";
  import ServiceWorkerRegistration from "$lib/components/ServiceWorkerRegistration.svelte";
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";

  let {
    data,
    children,
  }: {
    data: { auth?: AuthContext };
    children: import("svelte").Snippet;
  } = $props();

  $effect(() => {
    authState.setServerAuth(
      data.auth ?? {
        userId: null,
        authState: "anonymous",
        accessRole: "anonymous",
        email: null,
      },
    );
  });

  onMount(() => {
    void cleanupLegacyClientStorage();
    void authState.start();

    const onUnhandledRejection = (event: PromiseRejectionEvent) => {
      if (presentAuthRequiredNoticeIfNeeded(event.reason)) {
        event.preventDefault();
      }
    };
    window.addEventListener("unhandledrejection", onUnhandledRejection);
    return () =>
      window.removeEventListener("unhandledrejection", onUnhandledRejection);
  });

  /** Routes that own `mobileBottomBar` via local `$effect`; others default to section nav. */
  afterNavigate(({ to }) => {
    if (!to) return;
    const path = to.url.pathname;
    if (
      path === "/" ||
      path.startsWith("/channels/") ||
      path === "/highlights" ||
      path === "/vocabulary" ||
      path === "/download-queue" ||
      path === "/chat"
    ) {
      return;
    }
    mobileBottomBar.set({ kind: "sections" });
  });

  function confirmAuthRequiredSignIn() {
    const redirectTo = `${page.url.pathname}${page.url.search}`;
    dismissAuthRequiredNotice();
    void goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
  }
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
  {#if $authRequiredNotice}
    <SignInRequiredModal
      show={true}
      message={$authRequiredNotice}
      onConfirm={confirmAuthRequiredSignIn}
      onCancel={() => dismissAuthRequiredNotice()}
    />
  {/if}
  <div class="min-h-0 flex-1">
    {@render children()}
  </div>
  <AppBottomNav />
</div>
