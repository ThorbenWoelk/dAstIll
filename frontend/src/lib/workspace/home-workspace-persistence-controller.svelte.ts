import { replaceState as replacePageState } from "$app/navigation";
import { page } from "$app/state";
import { onMount } from "svelte";
import type { Component } from "svelte";

import { authState } from "$lib/auth-state.svelte";
import {
  loadWorkspaceState,
  restoreWorkspaceSnapshot,
  saveWorkspaceState,
  type WorkspaceStateSnapshot,
} from "$lib/channel-workspace";
import { getPreferences, savePreferences } from "$lib/api";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { mobileWorkspaceBrowseIntent } from "$lib/mobile-navigation/mobileWorkspaceBrowseIntent";
import { resolveBootstrapOnMount } from "$lib/ssr-bootstrap";
import {
  buildWorkspaceViewHref,
  mergeWorkspaceViewState,
  type WorkspaceViewState,
} from "$lib/view-url";
import { resolveAcknowledgedParam } from "$lib/workspace/types";
import {
  isWorkspaceContentMode,
  isWorkspaceVideoTypeFilter,
  type ChannelSortMode,
} from "$lib/workspace/types";
import type {
  AiStatus,
  ChannelSnapshot,
  SearchStatus,
  VideoTypeFilter,
} from "$lib/types";
import type { VocabularyReplacement } from "$lib/bindings/VocabularyReplacement";

import { createContentState } from "$lib/workspace/content-state.svelte";
import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";

export function createHomeWorkspacePersistenceController(options: {
  sidebarState: ReturnType<typeof createSidebarState>;
  content: ReturnType<typeof createContentState>;
  getWorkspaceStorageKey: () => string;
  getWorkspaceCacheScopeKey: () => string;
  getMobileViewportMq: () => boolean;
  setMobileViewportMq: (value: boolean) => void;
  getMobileBrowseOpen: () => boolean;
  setMobileBrowseOpen: (value: boolean) => void;
  getAiAvailable: () => boolean | null;
  setAiAvailable: (value: boolean | null) => void;
  getAiStatus: () => AiStatus | null;
  setAiStatus: (value: AiStatus | null) => void;
  getSearchStatus: () => SearchStatus | null;
  setSearchStatus: (value: SearchStatus | null) => void;
  getVocabularyReplacements: () => VocabularyReplacement[];
  setVocabularyReplacements: (value: VocabularyReplacement[]) => void;
  buildWorkspaceSnapshotCacheKey: (
    channelId: string,
    type: VideoTypeFilter,
    acknowledged: boolean | undefined,
  ) => string;
  restoreGuideFromUrl: () => void;
  applyChannelSnapshot: (
    channelId: string,
    snapshot: ChannelSnapshot,
    preferredVideoId: string | null,
    silent?: boolean,
  ) => Promise<void>;
  loadBootstrapRefresh: (options?: { silent?: boolean }) => Promise<void>;
}) {
  const { sidebarState, content } = options;

  let workspaceStateHydrated = $state(false);
  let shallowUrlSyncReady = $state(false);
  let viewUrlHydrated = $state(false);
  let preferencesHydrated = $state(false);
  let WorkspaceSearchBarComponent = $state<Component | null>(null);
  let preferencesSaveTimer: ReturnType<typeof setTimeout> | null = null;

  function restoreWorkspaceState() {
    const urlState: Partial<WorkspaceViewState> = {};
    if (page.data.selectedChannelId) {
      urlState.selectedChannelId = page.data.selectedChannelId;
    }
    if (page.data.selectedVideoId) {
      urlState.selectedVideoId = page.data.selectedVideoId;
    }
    if (page.data.contentMode) {
      urlState.contentMode = page.data.contentMode;
    }
    if (page.data.videoTypeFilter) {
      urlState.videoTypeFilter = page.data.videoTypeFilter;
    }
    if (page.data.acknowledgedFilter) {
      urlState.acknowledgedFilter = page.data.acknowledgedFilter;
    }

    const restored = mergeWorkspaceViewState(
      restoreWorkspaceSnapshot(
        typeof localStorage === "undefined"
          ? null
          : loadWorkspaceState(localStorage, options.getWorkspaceStorageKey()),
        {
          includeSelectedVideoId: true,
          includeContentMode: true,
          includeVideoTypeFilter: true,
          includeAcknowledgedFilter: true,
          includeChannelSortMode: true,
        },
      ),
      urlState,
    );

    sidebarState.applyRestoredSidebarState({
      ...("selectedChannelId" in restored
        ? { selectedChannelId: restored.selectedChannelId ?? null }
        : {}),
      ...("selectedVideoId" in restored
        ? { selectedVideoId: restored.selectedVideoId ?? null }
        : {}),
      ...(restored.videoTypeFilter &&
      isWorkspaceVideoTypeFilter(restored.videoTypeFilter)
        ? { videoTypeFilter: restored.videoTypeFilter }
        : {}),
      ...(restored.acknowledgedFilter
        ? { acknowledgedFilter: restored.acknowledgedFilter }
        : {}),
      ...(restored.channelSortMode
        ? { channelSortMode: restored.channelSortMode }
        : {}),
      ...(Array.isArray(restored.channelOrder)
        ? { channelOrder: restored.channelOrder }
        : {}),
    });
    if (restored.contentMode && isWorkspaceContentMode(restored.contentMode)) {
      content.setMode(restored.contentMode);
    }

    const url =
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- transient URL for one-time restore logic
      typeof window !== "undefined" ? new URL(window.location.href) : null;
    const videoInUrl = Boolean(url?.searchParams.get("video")?.trim());

    if (sidebarState.selectedVideoId) {
      const showVideoPanel = !options.getMobileViewportMq() || videoInUrl;
      options.setMobileBrowseOpen(!showVideoPanel);
    } else {
      options.setMobileBrowseOpen(true);
    }
  }

  function replaceWorkspaceUrl(href: string) {
    if (!shallowUrlSyncReady || typeof window === "undefined") return;
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- transient URL for navigation comparison
    const nextUrl = new URL(href, window.location.origin);
    if (
      nextUrl.pathname === window.location.pathname &&
      nextUrl.search === window.location.search
    ) {
      return;
    }
    replacePageState(
      `${nextUrl.pathname}${nextUrl.search}${nextUrl.hash}`,
      history.state,
    );
  }

  function persistViewUrl() {
    if (
      !viewUrlHydrated ||
      !shallowUrlSyncReady ||
      typeof window === "undefined"
    ) {
      return;
    }
    const omitVideoFromUrl =
      options.getMobileViewportMq() && options.getMobileBrowseOpen();
    const nextHref = buildWorkspaceViewHref({
      selectedChannelId: sidebarState.selectedChannelId,
      selectedVideoId: omitVideoFromUrl ? null : sidebarState.selectedVideoId,
      contentMode: content.contentMode,
      videoTypeFilter: sidebarState.videoTypeFilter,
      acknowledgedFilter: sidebarState.acknowledgedFilter,
    });
    replaceWorkspaceUrl(nextHref);
  }

  function persistWorkspaceState() {
    if (!workspaceStateHydrated || typeof localStorage === "undefined") return;
    const snapshot: WorkspaceStateSnapshot = {
      selectedChannelId: sidebarState.selectedChannelId,
      selectedVideoId: sidebarState.selectedVideoId,
      contentMode: content.contentMode,
      videoTypeFilter: sidebarState.videoTypeFilter,
      acknowledgedFilter: sidebarState.acknowledgedFilter,
      channelOrder: sidebarState.channelOrder,
      channelSortMode: sidebarState.channelSortMode,
    };
    saveWorkspaceState(
      localStorage,
      snapshot,
      options.getWorkspaceStorageKey(),
    );
    if (!preferencesHydrated) return;
    if (authState.current.authState !== "authenticated") return;
    if (preferencesSaveTimer) clearTimeout(preferencesSaveTimer);
    preferencesSaveTimer = setTimeout(() => {
      if (authState.current.authState !== "authenticated") {
        preferencesSaveTimer = null;
        return;
      }
      void savePreferences({
        channel_order: sidebarState.channelOrder,
        channel_sort_mode: sidebarState.channelSortMode as ChannelSortMode,
        vocabulary_replacements: options.getVocabularyReplacements(),
      }).catch((error) => {
        if (presentAuthRequiredNoticeIfNeeded(error)) return;
        throw error;
      });
      preferencesSaveTimer = null;
    }, 1000);
  }

  $effect(() => {
    persistWorkspaceState();
  });

  $effect(() => {
    persistViewUrl();
  });

  onMount(() => {
    const mq = window.matchMedia("(max-width: 1023px)");
    options.setMobileViewportMq(mq.matches);
    const onViewportChange = () => {
      options.setMobileViewportMq(mq.matches);
    };
    mq.addEventListener("change", onViewportChange);

    restoreWorkspaceState();
    options.restoreGuideFromUrl();
    const unsubscribeBrowseIntent = mobileWorkspaceBrowseIntent.subscribe(
      (wantsBrowse) => {
        if (!wantsBrowse) return;
        options.setMobileBrowseOpen(true);
        mobileWorkspaceBrowseIntent.set(false);
      },
    );
    workspaceStateHydrated = true;
    setTimeout(() => {
      shallowUrlSyncReady = true;
      persistViewUrl();
    }, 0);
    void (async () => {
      try {
        const selectedChannelIdAtMount = sidebarState.selectedChannelId;
        const selectedVideoIdAtMount = sidebarState.selectedVideoId;
        const acknowledgedAtMount = resolveAcknowledgedParam(
          sidebarState.acknowledgedFilter,
        );

        const [bootstrapResult, apiPreferences] = await Promise.all([
          resolveBootstrapOnMount({
            serverBootstrap: page.data.bootstrap ?? null,
            selectedChannelId: selectedChannelIdAtMount,
            workspaceCacheScopeKey: options.getWorkspaceCacheScopeKey(),
            viewSnapshotCacheKey: sidebarState.selectedChannelId
              ? options.buildWorkspaceSnapshotCacheKey(
                  sidebarState.selectedChannelId,
                  sidebarState.videoTypeFilter,
                  acknowledgedAtMount,
                )
              : null,
          }),
          getPreferences().catch(() => null),
        ]);

        if (apiPreferences && authState.current.authState === "authenticated") {
          sidebarState.applyChannelPreferencesState({
            channelOrder: apiPreferences.channel_order,
            channelSortMode:
              apiPreferences.channel_sort_mode as ChannelSortMode,
          });
          options.setVocabularyReplacements(
            apiPreferences.vocabulary_replacements ?? [],
          );
        }

        const hasInitialData = Boolean(
          bootstrapResult.channels && bootstrapResult.channels.length > 0,
        );

        if (bootstrapResult.channels && bootstrapResult.channels.length > 0) {
          sidebarState.applyLoadedChannelsState(
            bootstrapResult.channels,
            sidebarState.channelOrder,
          );
        }

        if (bootstrapResult.aiAvailable !== null) {
          options.setAiAvailable(bootstrapResult.aiAvailable);
        }
        if (bootstrapResult.aiStatus !== null) {
          options.setAiStatus(bootstrapResult.aiStatus);
        }
        if (bootstrapResult.searchStatus !== null) {
          options.setSearchStatus(bootstrapResult.searchStatus);
        }

        if (
          bootstrapResult.snapshot &&
          selectedChannelIdAtMount &&
          bootstrapResult.snapshot.channel_id === selectedChannelIdAtMount
        ) {
          await options.applyChannelSnapshot(
            selectedChannelIdAtMount,
            bootstrapResult.snapshot,
            selectedVideoIdAtMount,
            true,
          );
        }

        void options
          .loadBootstrapRefresh({ silent: hasInitialData })
          .finally(() => {
            viewUrlHydrated = true;
          });
      } finally {
        preferencesHydrated = true;
      }
    })();

    void import("$lib/components/workspace/WorkspaceSearchBar.svelte").then(
      (module) => {
        WorkspaceSearchBarComponent = module.default;
      },
    );

    return () => {
      mq.removeEventListener("change", onViewportChange);
      unsubscribeBrowseIntent();
      if (preferencesSaveTimer) {
        clearTimeout(preferencesSaveTimer);
        preferencesSaveTimer = null;
      }
    };
  });

  return {
    get WorkspaceSearchBarComponent() {
      return WorkspaceSearchBarComponent;
    },
    replaceWorkspaceUrl,
  };
}
