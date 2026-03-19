<script lang="ts">
  import type { ChatConversationSummary } from "$lib/types";
  import { formatRelativeTime } from "$lib/utils/date";

  let {
    conversations,
    activeConversationId = null,
    mobileVisible = false,
    loading = false,
    creating = false,
    onCreate = async () => {},
    onSelect = (_conversationId: string) => {},
    onRename = async (_conversationId: string, _title: string) => {},
    onDelete = async (_conversationId: string) => {},
  }: {
    conversations: ChatConversationSummary[];
    activeConversationId?: string | null;
    mobileVisible?: boolean;
    loading?: boolean;
    creating?: boolean;
    onCreate?: () => Promise<void> | void;
    onSelect?: (conversationId: string) => void;
    onRename?: (conversationId: string, title: string) => Promise<void> | void;
    onDelete?: (conversationId: string) => Promise<void> | void;
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
  class={`fade-in stagger-1 flex min-h-0 min-w-0 flex-col border-0 lg:h-full lg:gap-3 lg:border-r lg:border-[var(--accent-border-soft)] lg:px-5 ${mobileVisible ? "h-full gap-4 p-3" : "hidden lg:flex"}`}
>
  <div
    class="flex items-center justify-between gap-3 border-b border-[var(--accent-border-soft)] px-1 pb-3 pt-1"
  >
    <div>
      <p
        class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)]"
      >
        Conversations
      </p>
      <p class="mt-1 text-[13px] text-[var(--foreground)]">
        Persistent chat history
      </p>
    </div>
    <button
      type="button"
      class="inline-flex h-9 items-center justify-center rounded-full border border-[var(--accent)]/15 bg-[var(--accent-wash-strong)] px-3 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--accent-strong)] transition-colors hover:bg-[var(--accent)]/15 disabled:cursor-not-allowed disabled:opacity-55"
      disabled={creating}
      onclick={() => void onCreate()}
    >
      {creating ? "Creating…" : "New"}
    </button>
  </div>

  <div
    class="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-1 pb-4 pt-3 lg:pr-4"
  >
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
      <div class="space-y-2">
        {#each conversations as conversation (conversation.id)}
          <div
            class={`rounded-[var(--radius-md)] border p-3 transition-colors ${activeConversationId === conversation.id ? "border-[var(--accent)]/35 bg-[var(--accent-wash)]" : "border-transparent bg-transparent hover:border-[var(--accent-border-soft)] hover:bg-[var(--surface-frost)]"}`}
          >
            <div class="flex items-start gap-2">
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
                    class="truncate text-[12px] font-semibold text-[var(--foreground)]"
                  >
                    {conversation.title ?? "New conversation"}
                  </p>
                {/if}
                <div
                  class="mt-1 flex items-center gap-2 text-[10px] uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                >
                  <span>{formatRelativeTime(conversation.updated_at)}</span>
                  {#if conversation.title_status === "generating"}
                    <span
                      class="inline-flex items-center gap-1 text-[var(--accent-strong)]"
                    >
                      <span
                        class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--accent)]"
                      ></span>
                      naming
                    </span>
                  {/if}
                </div>
              </button>

              {#if editingConversationId !== conversation.id}
                <div class="flex shrink-0 items-center gap-1">
                  <button
                    type="button"
                    class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
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
                  <button
                    type="button"
                    class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
                    aria-label="Delete conversation"
                    onclick={() => void onDelete(conversation.id)}
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
                      <path d="M3 6h18" />
                      <path d="M8 6V4h8v2" />
                      <path d="m19 6-1 14H6L5 6" />
                    </svg>
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</aside>
