<script lang="ts">
  import "../app.css";
  import { afterNavigate, goto } from "$app/navigation";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import type { AuthContext } from "$lib/auth";
  import { assertHostedApiBaseConfigured } from "$lib/api/client";
  import {
    cleanupLegacyClientStorage,
    getAuthStorageScopeKey,
    getScopedStorageKey,
  } from "$lib/auth/storage";
  import { authState } from "$lib/auth/state.svelte";
  import {
    authRequiredNotice,
    dismissAuthRequiredNotice,
    presentAuthRequiredNoticeIfNeeded,
  } from "$lib/auth/required-notice";
  import SignInRequiredModal from "$lib/components/SignInRequiredModal.svelte";
  import GlobalKeyboardShortcuts from "$lib/components/GlobalKeyboardShortcuts.svelte";
  import MobileViewportInset from "$lib/components/MobileViewportInset.svelte";
  import MobileSectionDrawer from "$lib/components/mobile/MobileSectionDrawer.svelte";
  import ServiceWorkerRegistration from "$lib/components/ServiceWorkerRegistration.svelte";
  import { applyStoredTheme } from "$lib/platform/theme";

  let {
    data,
    children,
  }: {
    data: { auth?: AuthContext };
    children: import("svelte").Snippet;
  } = $props();

  let themeMediaQuery = $state<MediaQueryList | null>(null);
  let themeStorageKey = $derived(
    getScopedStorageKey(
      "dastill-theme-appearance",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  let colorStorageKey = $derived(
    getScopedStorageKey(
      "dastill-theme-color",
      getAuthStorageScopeKey(authState.current),
    ),
  );

  function syncTheme() {
    if (typeof window === "undefined") {
      return;
    }

    applyStoredTheme(
      document,
      window.localStorage,
      themeMediaQuery?.matches ??
        window.matchMedia("(prefers-color-scheme: dark)").matches,
      {
        themeKey: themeStorageKey,
        colorKey: colorStorageKey,
      },
    );
  }

  $effect(() => {
    if (data.auth) {
      authState.setServerAuth(data.auth);
    }
  });

  $effect(() => {
    authState.current;
    syncTheme();
  });

  onMount(() => {
    assertHostedApiBaseConfigured();
    void cleanupLegacyClientStorage();
    void authState.start();
    themeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    syncTheme();

    const onThemeChange = () => {
      syncTheme();
    };

    const onUnhandledRejection = (event: PromiseRejectionEvent) => {
      if (presentAuthRequiredNoticeIfNeeded(event.reason)) {
        event.preventDefault();
      }
    };
    themeMediaQuery.addEventListener("change", onThemeChange);
    window.addEventListener("unhandledrejection", onUnhandledRejection);
    return () => {
      themeMediaQuery?.removeEventListener("change", onThemeChange);
      window.removeEventListener("unhandledrejection", onUnhandledRejection);
    };
  });

  /** Route changes */
  afterNavigate(({ to }) => {
    if (!to) return;
  });

  let sectionDrawerOpen = $state(false);
  let showDrawer = $derived(
    !page.url.pathname.startsWith("/login") &&
      !page.url.pathname.startsWith("/logout"),
  );

  $effect(() => {
    if (typeof window === "undefined") return;

    function handleOpenDrawer() {
      sectionDrawerOpen = true;
    }
    window.addEventListener("dastill:open-section-drawer", handleOpenDrawer);
    return () =>
      window.removeEventListener(
        "dastill:open-section-drawer",
        handleOpenDrawer,
      );
  });

  // Close drawer on navigation
  $effect(() => {
    page.url.pathname;
    sectionDrawerOpen = false;
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
  {#if showDrawer}
    <MobileSectionDrawer
      open={sectionDrawerOpen}
      onClose={() => {
        sectionDrawerOpen = false;
      }}
    />
  {/if}
</div>
