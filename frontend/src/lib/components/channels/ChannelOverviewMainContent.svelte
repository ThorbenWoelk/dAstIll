<script lang="ts">
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import { formatShortDate } from "$lib/utils/date";
  import type { Channel } from "$lib/types";

  let {
    selectedChannel = null,
    loadingOverview = false,
    missingChannelMessage = null,
    earliestSyncDateInput = $bindable(""),
    savingSyncDate = false,
    onSaveSyncDate = () => {},
    onBack = () => {},
    onOpenChannels = () => {},
  }: {
    selectedChannel?: Channel | null;
    loadingOverview?: boolean;
    missingChannelMessage?: string | null;
    earliestSyncDateInput?: string;
    savingSyncDate?: boolean;
    onSaveSyncDate?: () => void;
    onBack?: () => void;
    onOpenChannels?: () => void;
  } = $props();
</script>

<section
  id="content-view"
  class="fade-in stagger-3 relative z-10 flex h-full min-h-0 min-w-0 flex-col overflow-hidden lg:gap-4 lg:px-8 lg:pt-6 lg:pb-6"
>
  <div
    class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--accent-border-soft)] px-4 py-4 sm:px-6 lg:px-0"
  >
    <div class="flex min-w-0 items-center gap-2 sm:gap-4">
      <button
        type="button"
        class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-70 transition hover:bg-[var(--accent-wash)] hover:opacity-100 lg:hidden"
        aria-label="Back to workspace"
        onclick={onBack}
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M15 18l-6-6 6-6" />
        </svg>
      </button>
      <button
        type="button"
        class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-70 transition hover:bg-[var(--accent-wash)] hover:opacity-100 lg:hidden"
        aria-label="Open channel list"
        onclick={onOpenChannels}
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
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
        class="h-14 w-14 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
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
          class="hidden text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50 sm:block"
        >
          Workspace
        </p>
        <h1
          class="mt-1 font-serif text-[22px] font-bold tracking-tight text-[var(--foreground)] sm:mt-2 sm:text-[32px]"
        >
          {selectedChannel ? selectedChannel.name : "Channel overview"}
        </h1>
        <p
          class="mt-1 text-[13px] text-[var(--soft-foreground)] sm:mt-2 sm:text-[14px]"
        >
          {#if selectedChannel}
            {selectedChannel.handle ?? selectedChannel.id}
          {:else}
            Follow channels and tune sync boundaries from the shared app view.
          {/if}
        </p>
      </div>
    </div>
  </div>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding min-h-0 flex-1 overflow-y-auto px-4 py-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
  >
    {#if loadingOverview}
      <div
        class="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(18rem,0.8fr)]"
      >
        <div class="space-y-4">
          {#each Array.from({ length: 1 }) as _, index (index)}
            <div
              class="animate-pulse rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-5 shadow-sm"
            >
              <div
                class="h-4 w-28 rounded-full bg-[var(--border)] opacity-60"
              ></div>
              <div
                class="mt-4 h-10 w-3/4 rounded-full bg-[var(--border)] opacity-35"
              ></div>
              <div
                class="mt-3 h-3 w-full rounded-full bg-[var(--border)] opacity-25"
              ></div>
              <div
                class="mt-2 h-3 w-2/3 rounded-full bg-[var(--border)] opacity-20"
              ></div>
            </div>
          {/each}
        </div>

        <div class="space-y-4">
          <div
            class="animate-pulse rounded-[var(--radius-lg)] bg-[var(--surface)] p-5 shadow-sm"
          >
            <div
              class="h-4 w-24 rounded-full bg-[var(--border)] opacity-60"
            ></div>
            <div
              class="mt-4 h-3 w-1/2 rounded-full bg-[var(--border)] opacity-25"
            ></div>
            <div
              class="mt-2 h-3 w-2/3 rounded-full bg-[var(--border)] opacity-20"
            ></div>
          </div>
        </div>
      </div>
    {:else if missingChannelMessage}
      <div
        class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-6 shadow-sm"
      >
        <p
          class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50"
        >
          Channel overview
        </p>
        <p class="mt-3 text-[16px] font-semibold text-[var(--foreground)]">
          {missingChannelMessage}
        </p>
      </div>
    {:else if selectedChannel}
      <div
        class="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(18rem,0.8fr)]"
      >
        <div class="space-y-4">
          <section
            class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-5 shadow-sm sm:p-6"
          >
            <p
              class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
            >
              Sync boundary
            </p>
            <p
              class="mt-3 max-w-2xl text-[14px] leading-6 text-[var(--soft-foreground)]"
            >
              Control how far back this channel should sync inside the shared
              workspace. Newer videos stay surfaced automatically once
              transcripts are ready.
            </p>
            <div class="mt-4 flex flex-col gap-3 sm:flex-row sm:items-center">
              <input
                type="date"
                class="min-w-0 flex-1 rounded-full border border-[var(--accent-border-soft)] bg-[var(--surface)] px-4 py-2 text-[14px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                bind:value={earliestSyncDateInput}
                disabled={savingSyncDate}
              />
              <button
                type="button"
                class="inline-flex items-center justify-center rounded-full bg-[var(--foreground)] px-4 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-40"
                onclick={onSaveSyncDate}
                disabled={!earliestSyncDateInput || savingSyncDate}
              >
                {savingSyncDate ? "Saving" : "Save"}
              </button>
            </div>
          </section>
        </div>

        <aside class="space-y-4">
          <section
            class="rounded-[var(--radius-lg)] bg-[var(--surface)] p-5 shadow-sm"
          >
            <p
              class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
            >
              Details
            </p>

            <dl class="mt-4 space-y-4 text-[14px]">
              <div>
                <dt class="text-[var(--soft-foreground)] opacity-70">Handle</dt>
                <dd class="mt-1 font-medium text-[var(--foreground)]">
                  {selectedChannel.handle ?? "Not provided"}
                </dd>
              </div>

              <div>
                <dt class="text-[var(--soft-foreground)] opacity-70">
                  Channel ID
                </dt>
                <dd class="mt-1 break-all font-medium text-[var(--foreground)]">
                  {selectedChannel.id}
                </dd>
              </div>

              <div>
                <dt class="text-[var(--soft-foreground)] opacity-70">Added</dt>
                <dd class="mt-1 font-medium text-[var(--foreground)]">
                  {formatShortDate(selectedChannel.added_at)}
                </dd>
              </div>

              <div>
                <dt class="text-[var(--soft-foreground)] opacity-70">
                  Boundary source
                </dt>
                <dd class="mt-1 font-medium text-[var(--foreground)]">
                  {selectedChannel.earliest_sync_date_user_set
                    ? "Manual override"
                    : "Derived from ready transcripts"}
                </dd>
              </div>
            </dl>
          </section>
        </aside>
      </div>
    {/if}
  </div>
</section>
