<script lang="ts">
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import { formatShortDate } from "$lib/utils/date";
  import type { Channel } from "$lib/types";

  let {
    selectedChannel = null,
    loadingOverview = false,
    missingChannelMessage = null,
    earliestSyncDateInput = $bindable(""),
    savingSyncDate = false,
    canDeleteChannel = false,
    onSaveSyncDate = () => {},
    onDeleteChannel = () => {},
    onBack = () => {},
    onOpenChannels = () => {},
  }: {
    selectedChannel?: Channel | null;
    loadingOverview?: boolean;
    missingChannelMessage?: string | null;
    earliestSyncDateInput?: string;
    savingSyncDate?: boolean;
    canDeleteChannel?: boolean;
    onSaveSyncDate?: () => void;
    onDeleteChannel?: () => void;
    onBack?: () => void;
    onOpenChannels?: () => void;
  } = $props();
</script>

<section
  id="content-view"
  class="fade-in stagger-3 relative z-10 flex h-full min-h-0 min-w-0 flex-col overflow-hidden lg:gap-6 lg:px-8 lg:pt-8 lg:pb-6"
>
  <div
    class="flex flex-wrap items-start justify-between gap-4 px-4 py-4 sm:px-6 lg:px-0 lg:py-0"
  >
    <div class="flex min-w-0 items-center gap-3 sm:gap-4">
      <button
        type="button"
        class="inline-flex h-9 w-9 items-center justify-center rounded-md text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface)] hover:text-[var(--foreground)] lg:hidden"
        aria-label="Back to workspace"
        onclick={onBack}
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.75"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M15 18l-6-6 6-6" />
        </svg>
      </button>
      <button
        type="button"
        class="inline-flex h-9 w-9 items-center justify-center rounded-md text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface)] hover:text-[var(--foreground)] lg:hidden"
        aria-label="Open channel list"
        onclick={onOpenChannels}
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.75"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4 7h16" />
          <path d="M4 12h16" />
          <path d="M4 17h16" />
        </svg>
      </button>
      <div
        class="h-12 w-12 shrink-0 overflow-hidden rounded-full bg-[var(--muted)] sm:h-14 sm:w-14"
      >
        {#if selectedChannel}
          <img
            src={selectedChannel.thumbnail_url || defaultChannelIcon}
            alt={selectedChannel.name}
            class="h-full w-full object-cover"
            referrerpolicy="no-referrer"
          />
        {/if}
      </div>

      <div class="min-w-0">
        <p
          class="hidden text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--soft-foreground)] sm:block"
        >
          Workspace
        </p>
        <h1
          class="mt-0.5 font-serif text-[22px] font-bold tracking-tight text-[var(--foreground)] sm:mt-1 sm:text-[28px]"
        >
          {selectedChannel ? selectedChannel.name : "Channel overview"}
        </h1>
        {#if selectedChannel}
          <p class="mt-0.5 text-[13px] text-[var(--soft-foreground)]">
            {selectedChannel.handle ?? selectedChannel.id}
          </p>
        {:else}
          <p class="mt-0.5 text-[13px] text-[var(--soft-foreground)]">
            Follow channels and tune sync boundaries from the shared app view.
          </p>
        {/if}
      </div>
    </div>
    {#if selectedChannel && canDeleteChannel}
      <button
        type="button"
        class="inline-flex items-center justify-center gap-2 rounded-md border border-[var(--danger)]/25 px-3 py-1.5 text-xs font-medium text-[var(--danger)] transition-colors hover:bg-[var(--danger)]/10"
        onclick={onDeleteChannel}
        aria-label="Delete channel"
      >
        <TrashIcon size={13} strokeWidth={1.75} />
        Delete channel
      </button>
    {/if}
  </div>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding min-h-0 flex-1 overflow-y-auto px-4 py-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0 lg:pt-0"
  >
    {#if loadingOverview}
      <div class="max-w-3xl space-y-10 animate-pulse">
        <div>
          <div class="h-4 w-40 rounded bg-[var(--border)] opacity-50"></div>
          <div
            class="mt-3 h-3 w-3/4 rounded bg-[var(--border)] opacity-25"
          ></div>
          <div
            class="mt-4 h-9 w-1/2 rounded bg-[var(--border)] opacity-30"
          ></div>
        </div>
        <hr class="border-[var(--border-soft)]" />
        <div>
          <div class="h-4 w-24 rounded bg-[var(--border)] opacity-50"></div>
          <div
            class="mt-3 h-3 w-1/2 rounded bg-[var(--border)] opacity-25"
          ></div>
          <div
            class="mt-2 h-3 w-2/3 rounded bg-[var(--border)] opacity-20"
          ></div>
        </div>
      </div>
    {:else if missingChannelMessage}
      <div class="max-w-2xl">
        <h4 class="text-sm font-semibold text-[var(--foreground)]">
          Channel overview
        </h4>
        <p class="mt-2 text-[13px] text-[var(--soft-foreground)]">
          {missingChannelMessage}
        </p>
      </div>
    {:else if selectedChannel}
      <div class="max-w-3xl space-y-10">
        <section id="sync-boundary">
          <h4 class="text-sm font-semibold text-[var(--foreground)]">
            Sync boundary
          </h4>
          <p
            class="mt-1 max-w-xl text-[13px] leading-6 text-[var(--soft-foreground)]"
          >
            Control how far back this channel should sync inside the shared
            workspace. Newer videos stay surfaced automatically once transcripts
            are ready.
          </p>
          <div class="mt-4 flex flex-col gap-2 sm:flex-row sm:items-center">
            <input
              type="date"
              class="min-w-0 max-w-xs flex-1 rounded-md border border-[var(--border-soft)] bg-[var(--background)] px-3 py-1.5 text-[13px] font-medium text-[var(--foreground)] transition-colors focus:border-[var(--accent)]/50 focus:outline-none"
              bind:value={earliestSyncDateInput}
              disabled={savingSyncDate}
            />
            <button
              type="button"
              class="inline-flex items-center justify-center self-start rounded-md bg-[var(--foreground)] px-3 py-1.5 text-[12px] font-semibold text-[var(--background)] transition-colors hover:bg-[var(--accent-strong)] disabled:opacity-40"
              onclick={onSaveSyncDate}
              disabled={!earliestSyncDateInput || savingSyncDate}
            >
              {savingSyncDate ? "Saving" : "Save"}
            </button>
          </div>
        </section>

        <hr class="border-[var(--border-soft)]" />

        <section>
          <h4 class="text-sm font-semibold text-[var(--foreground)]">
            Details
          </h4>
          <dl class="mt-4 grid gap-4 text-[13px] sm:grid-cols-2 sm:gap-x-8">
            <div>
              <dt class="text-[var(--soft-foreground)]">Handle</dt>
              <dd class="mt-1 font-medium text-[var(--foreground)]">
                {selectedChannel.handle ?? "Not provided"}
              </dd>
            </div>

            <div>
              <dt class="text-[var(--soft-foreground)]">Channel ID</dt>
              <dd class="mt-1 break-all font-medium text-[var(--foreground)]">
                {selectedChannel.id}
              </dd>
            </div>

            <div>
              <dt class="text-[var(--soft-foreground)]">Added</dt>
              <dd class="mt-1 font-medium text-[var(--foreground)]">
                {formatShortDate(selectedChannel.added_at)}
              </dd>
            </div>

            <div>
              <dt class="text-[var(--soft-foreground)]">Boundary source</dt>
              <dd class="mt-1 font-medium text-[var(--foreground)]">
                {selectedChannel.earliest_sync_date_user_set
                  ? "Manual override"
                  : "Derived from ready transcripts"}
              </dd>
            </div>
          </dl>
        </section>
      </div>
    {/if}
  </div>
</section>
