<script lang="ts">
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import type { ChatConversationSummary } from "$lib/types";

  let {
    conversations,
    activeConversationId = null,
    mobileVisible = false,
    loading = false,
    creating = false,
    deletingAll = false,
    canDelete = false,
    onCreate = async () => {},
    onSelect = (_conversationId: string) => {},
    onRename = async (_conversationId: string, _title: string) => {},
    onDelete = async (_conversationId: string) => {},
    onDeleteAll = async () => {},
  }: {
    conversations: ChatConversationSummary[];
    activeConversationId?: string | null;
    mobileVisible?: boolean;
    loading?: boolean;
    creating?: boolean;
    deletingAll?: boolean;
    canDelete?: boolean;
    onCreate?: () => Promise<void> | void;
    onSelect?: (conversationId: string) => void;
    onRename?: (conversationId: string, title: string) => Promise<void> | void;
    onDelete?: (conversationId: string) => Promise<void> | void;
    onDeleteAll?: () => Promise<void> | void;
  } = $props();

  let editingConversationId = $state<string | null>(null);
  let editingTitle = $state("");

  function beginRename(conversation: ChatConversationSummary) {
    editingConversationId = conversation.id;
    editingTitle = conversation.title ?? "";
  }

  async function submitRename(conversationId: string) {
    const trimmed = editingTitle.trim();
    if (!trimmed) {
      editingConversationId = null;
      editingTitle = "";
      return;
    }
    await onRename(conversationId, trimmed);
    editingConversationId = null;
    editingTitle = "";
  }
</script>

<aside
  class={`fade-in stagger-1 flex min-h-0 min-w-0 flex-col border-0 lg:h-full lg:border-r lg:border-[var(--border-soft)] lg:bg-[var(--muted)] ${mobileVisible ? "h-full gap-3 p-3" : "hidden lg:flex"}`}
>
  <div class="shrink-0 border-b border-[var(--border-soft)] p-4">
    <button
      type="button"
      class="flex w-full items-center justify-center gap-2 rounded-[var(--radius-md)] bg-[var(--foreground)] px-4 py-2 text-[13px] font-semibold text-[var(--background)] transition-colors hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-55"
      aria-label="New"
      disabled={creating}
      onclick={() => void onCreate()}
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M12 5v14" />
        <path d="M5 12h14" />
      </svg>
      {creating ? "Creating…" : "New Research"}
    </button>
  </div>

  <div
    class="custom-scrollbar flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-3"
  >
    <div
      class="flex items-center justify-between px-2 pb-2 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)]"
    >
      <span>Recent Sessions</span>
      {#if canDelete && conversations.length > 0}
        <button
          type="button"
          class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] transition-colors hover:text-[var(--danger)] disabled:cursor-not-allowed disabled:opacity-55"
          aria-label="Delete all conversations"
          disabled={deletingAll}
          onclick={() => void onDeleteAll()}
        >
          {deletingAll ? "Deleting…" : "Clear"}
        </button>
      {/if}
    </div>
    {#if loading}
      <div
        class="flex h-full min-h-[14rem] items-center justify-center text-[12px] text-[var(--soft-foreground)]"
      >
        Loading conversations…
      </div>
    {:else if conversations.length === 0}
      <div
        class="flex h-full min-h-[14rem] items-center justify-center px-4 text-center text-[12px] leading-relaxed text-[var(--soft-foreground)]"
      >
        Start a new conversation to ask grounded questions about your library.
      </div>
    {:else}
      <div class="flex flex-col gap-0.5">
        {#each conversations as conversation (conversation.id)}
          <div
            class={`group/row rounded-[var(--radius-md)] px-2 py-1.5 transition-colors ${activeConversationId === conversation.id ? "bg-[var(--accent-wash-strong)]" : "bg-transparent hover:bg-[var(--accent-wash)]"}`}
          >
            <div class="flex items-center gap-1">
              <button
                type="button"
                class="min-w-0 flex-1 text-left"
                onclick={() => onSelect(conversation.id)}
              >
                {#if editingConversationId === conversation.id}
                  <input
                    bind:value={editingTitle}
                    class="w-full rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--background)] px-2 py-1 text-[12px] font-semibold text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/35"
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void submitRename(conversation.id);
                      } else if (event.key === "Escape") {
                        editingConversationId = null;
                        editingTitle = "";
                      }
                    }}
                    onblur={() => void submitRename(conversation.id)}
                  />
                {:else}
                  <p
                    class={`truncate text-[13px] ${activeConversationId === conversation.id ? "font-semibold text-[var(--foreground)]" : "font-medium text-[var(--soft-foreground)] group-hover/row:text-[var(--foreground)]"}`}
                  >
                    {conversation.title ?? "New conversation"}
                  </p>
                {/if}
                {#if conversation.title_status === "generating"}
                  <div
                    class="mt-0.5 flex items-center gap-1 text-[10px] uppercase tracking-[0.1em] text-[var(--accent-strong)]"
                  >
                    <span
                      class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--accent)]"
                    ></span>
                    naming
                  </div>
                {/if}
              </button>

              {#if editingConversationId !== conversation.id}
                <div
                  class="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover/row:opacity-100 focus-within:opacity-100"
                >
                  <button
                    type="button"
                    class="inline-flex h-6 w-6 items-center justify-center rounded text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface-strong)] hover:text-[var(--foreground)]"
                    aria-label="Rename conversation"
                    onclick={() => beginRename(conversation)}
                  >
                    <svg
                      width="12"
                      height="12"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      <path d="M12 20h9" />
                      <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
                    </svg>
                  </button>
                  {#if canDelete}
                    <button
                      type="button"
                      class="inline-flex h-6 w-6 items-center justify-center rounded text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface-strong)] hover:text-[var(--danger)]"
                      aria-label="Delete conversation"
                      onclick={() => void onDelete(conversation.id)}
                    >
                      <TrashIcon size={12} strokeWidth={2} />
                    </button>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</aside>
