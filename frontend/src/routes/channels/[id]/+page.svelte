<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import {
    getChannelSyncDepth,
    listChannels,
    refreshChannel,
    updateChannel,
  } from "$lib/api";
  import type { Channel, SyncDepth } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";

  let channel = $state<Channel | null>(null);
  let syncDepth = $state<SyncDepth | null>(null);
  let earliestSyncDateInput = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let errorMessage = $state<string | null>(null);

  function resolveEffectiveSyncDate(
    currentChannel: Channel | null,
    currentSyncDepth: SyncDepth | null,
  ) {
    if (!currentChannel) {
      return null;
    }

    return currentChannel.earliest_sync_date_user_set
      ? currentChannel.earliest_sync_date
      : (currentSyncDepth?.derived_earliest_ready_date ??
          currentChannel.earliest_sync_date ??
          null);
  }

  function syncInputValue() {
    const effective = resolveEffectiveSyncDate(channel, syncDepth);
    earliestSyncDateInput = effective
      ? new Date(effective).toISOString().split("T")[0]
      : "";
  }

  async function loadChannelOverview() {
    loading = true;
    errorMessage = null;

    try {
      const channelId = $page.params.id ?? "";
      if (!channelId) {
        channel = null;
        syncDepth = null;
        earliestSyncDateInput = "";
        errorMessage = "Channel not found.";
        return;
      }

      const allChannels = await listChannels();
      channel = allChannels.find((item) => item.id === channelId) ?? null;

      if (!channel) {
        errorMessage = "Channel not found.";
        syncDepth = null;
        earliestSyncDateInput = "";
        return;
      }

      syncDepth = await getChannelSyncDepth(channelId);
      syncInputValue();
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loading = false;
    }
  }

  async function saveSyncDate() {
    if (!$page.params.id || !earliestSyncDateInput || saving) {
      return;
    }

    saving = true;
    errorMessage = null;

    try {
      channel = await updateChannel($page.params.id, {
        earliest_sync_date: new Date(earliestSyncDateInput).toISOString(),
        earliest_sync_date_user_set: true,
      });
      await refreshChannel($page.params.id);
      syncDepth = await getChannelSyncDepth($page.params.id);
      syncInputValue();
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    void loadChannelOverview();
  });
</script>

<div class="min-h-screen bg-[var(--background)] px-4 py-6 sm:px-6 lg:px-8">
  <div class="mx-auto max-w-3xl">
    <button
      type="button"
      class="inline-flex items-center gap-2 rounded-full px-3 py-2 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
      onclick={() => void goto("/")}
    >
      <svg
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <polyline points="15 18 9 12 15 6" />
      </svg>
      Workspace
    </button>

    <section
      class="mt-6 rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-5 shadow-sm sm:p-6"
    >
      {#if loading}
        <div class="space-y-3 animate-pulse">
          <div
            class="h-4 w-28 rounded-full bg-[var(--border)] opacity-50"
          ></div>
          <div
            class="h-12 w-full rounded-[var(--radius-md)] bg-[var(--border)] opacity-30"
          ></div>
          <div
            class="h-24 w-full rounded-[var(--radius-md)] bg-[var(--border)] opacity-20"
          ></div>
        </div>
      {:else if !channel}
        <div class="space-y-2">
          <p
            class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
          >
            Channel overview
          </p>
          <p class="text-[16px] font-semibold text-[var(--foreground)]">
            {errorMessage ?? "Channel not found."}
          </p>
        </div>
      {:else}
        <div class="flex items-start gap-4">
          <div
            class="h-14 w-14 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
          >
            <img
              src={channel.thumbnail_url || defaultChannelIcon}
              alt={channel.name}
              class="h-full w-full object-cover"
              referrerpolicy="no-referrer"
            />
          </div>
          <div class="min-w-0 flex-1">
            <p
              class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
            >
              Channel overview
            </p>
            <h1
              class="mt-1 text-[24px] font-semibold tracking-tight text-[var(--foreground)]"
            >
              {channel.name}
            </h1>
            <p
              class="mt-1 text-[13px] text-[var(--soft-foreground)] opacity-70"
            >
              {channel.handle ?? channel.id}
            </p>
          </div>
        </div>

        <div class="mt-8 grid gap-4">
          <div class="rounded-[var(--radius-md)] bg-[var(--surface)] px-4 py-4">
            <p
              class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
            >
              Current sync boundary
            </p>
            <p class="mt-2 text-[15px] font-semibold text-[var(--foreground)]">
              {formatSyncDate(resolveEffectiveSyncDate(channel, syncDepth))}
            </p>
            <p class="mt-2 text-[13px] leading-6 text-[var(--soft-foreground)]">
              Choose how far back this channel should sync in the workspace.
            </p>
          </div>

          <div class="rounded-[var(--radius-md)] bg-[var(--surface)] px-4 py-4">
            <div class="flex items-center gap-2">
              <input
                type="date"
                class="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-2.5 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                bind:value={earliestSyncDateInput}
                disabled={saving}
              />
              <button
                type="button"
                class="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
                onclick={() => void saveSyncDate()}
                disabled={!earliestSyncDateInput || saving}
              >
                {saving ? "..." : "Set"}
              </button>
            </div>
          </div>

          {#if errorMessage}
            <p class="text-[13px] text-[var(--danger)]">
              {errorMessage}
            </p>
          {/if}
        </div>
      {/if}
    </section>
  </div>
</div>
