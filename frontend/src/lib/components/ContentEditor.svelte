<script lang="ts">
  import ContentActionButton from "./ContentActionButton.svelte";

  let {
    value = "",
    editing = false,
    busy = false,
    formatting = false,
    regenerating = false,
    reverting = false,
    showFormatAction = false,
    showRegenerateAction = false,
    showRevertAction = false,
    showEditAction = true,
    canRevert = true,
    youtubeUrl = null,
    onEdit = () => {},
    onCancel = () => {},
    onSave = () => {},
    onFormat = () => {},
    onRegenerate = () => {},
    onRevert = () => {},
    onChange = (_: string) => {},
    onAcknowledgeToggle = undefined,
    acknowledged = false,
    aiAvailable = true,
  }: {
    value?: string;
    editing?: boolean;
    busy?: boolean;
    formatting?: boolean;
    regenerating?: boolean;
    reverting?: boolean;
    showFormatAction?: boolean;
    showRegenerateAction?: boolean;
    showRevertAction?: boolean;
    showEditAction?: boolean;
    canRevert?: boolean;
    youtubeUrl?: string | null;
    onEdit?: () => void;
    onCancel?: () => void;
    onSave?: () => void;
    onFormat?: () => void;
    onRegenerate?: () => void;
    onRevert?: () => void;
    onChange?: (next: string) => void;
    onAcknowledgeToggle?: (() => void) | undefined;
    acknowledged?: boolean;
    aiAvailable?: boolean;
  } = $props();

  let formatActionLabel = $derived(
    formatting
      ? "Formatting transcript"
      : aiAvailable
        ? "Clean formatting"
        : "auto-format (AI engine required)",
  );
  let revertActionLabel = $derived(
    reverting ? "Reverting transcript" : "Revert to original transcript",
  );
  let regenerateActionLabel = $derived(
    regenerating
      ? "Regenerating summary"
      : aiAvailable
        ? "Regenerate summary"
        : "regenerate (AI engine required)",
  );
</script>

{#if editing}
  <div class="flex flex-col gap-4 fade-in">
    <div class="flex flex-wrap items-center gap-3">
      {#if showFormatAction}
        <ContentActionButton
          icon="format"
          loading={formatting}
          disabled={busy || formatting || reverting || !aiAvailable}
          label={formatActionLabel}
          tooltip={formatting ? "Formatting…" : formatActionLabel}
          onClick={onFormat}
        />
        {#if showRevertAction}
          <ContentActionButton
            icon="revert"
            loading={reverting}
            disabled={busy ||
              formatting ||
              regenerating ||
              reverting ||
              !canRevert}
            label={revertActionLabel}
            tooltip={reverting ? "Reverting…" : revertActionLabel}
            onClick={onRevert}
          />
        {/if}
      {/if}
      {#if showRegenerateAction}
        <ContentActionButton
          icon="regenerate"
          loading={regenerating}
          disabled={busy ||
            formatting ||
            regenerating ||
            reverting ||
            !aiAvailable}
          label={regenerateActionLabel}
          tooltip={regenerating ? "Regenerating…" : regenerateActionLabel}
          onClick={onRegenerate}
        />
      {/if}
      {#if youtubeUrl}
        <ContentActionButton
          icon="youtube"
          href={youtubeUrl}
          label="Open video on YouTube"
          tooltip="Open on YouTube"
        />
      {/if}
      <div class="ml-auto flex items-center gap-2">
        <button
          type="button"
          class="rounded-full bg-[var(--foreground)] px-5 py-2 text-[11px] font-bold uppercase tracking-[0.1em] text-white transition-all hover:bg-[var(--accent-strong)] disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={onSave}
          disabled={busy}
        >
          {busy ? "Saving..." : "Save"}
        </button>
        <button
          type="button"
          class="rounded-full px-4 py-2 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] transition-all hover:text-[var(--foreground)] disabled:opacity-20 focus-visible:outline-none"
          onclick={onCancel}
          disabled={busy}
        >
          Cancel
        </button>
      </div>
    </div>
    <textarea
      name="content"
      autocomplete="off"
      aria-label="Content editor"
      class={`min-h-[400px] w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] p-8 text-[15px] font-medium leading-[1.7] shadow-sm transition-all focus-within:ring-2 focus-within:ring-[var(--accent)]/10 focus-within:border-[var(--accent)]/40 focus-visible:outline-none max-lg:border-0 max-lg:bg-transparent max-lg:p-0 max-lg:shadow-none ${
        formatting
          ? "opacity-50 blur-[0.5px] bg-[var(--background)]"
          : "bg-[var(--surface)]"
      }`}
      {value}
      oninput={(event) =>
        onChange((event.currentTarget as HTMLTextAreaElement).value)}
      placeholder="Refine the distillation here…"
    ></textarea>
  </div>
{:else}
  <div class="flex flex-wrap items-center gap-4">
    <div class="flex flex-wrap items-center gap-4">
      {#if showFormatAction}
        <ContentActionButton
          compact
          icon="format"
          loading={formatting}
          disabled={busy || formatting || reverting || !aiAvailable}
          label={formatActionLabel}
          tooltip={formatting ? "Formatting…" : formatActionLabel}
          onClick={onFormat}
        />
        {#if showRevertAction}
          <ContentActionButton
            compact
            icon="revert"
            loading={reverting}
            disabled={busy || formatting || reverting || !canRevert}
            label={revertActionLabel}
            tooltip={reverting ? "Reverting…" : revertActionLabel}
            onClick={onRevert}
          />
        {/if}
      {/if}
      {#if showRegenerateAction}
        <ContentActionButton
          compact
          icon="regenerate"
          loading={regenerating}
          disabled={busy ||
            formatting ||
            regenerating ||
            reverting ||
            !aiAvailable}
          label={regenerateActionLabel}
          tooltip={regenerating ? "Regenerating…" : regenerateActionLabel}
          onClick={onRegenerate}
        />
      {/if}
    </div>

    <div class="ml-auto flex flex-wrap items-center gap-4">
      {#if youtubeUrl}
        <ContentActionButton
          compact
          icon="youtube"
          href={youtubeUrl}
          label="Open video on YouTube"
          tooltip="Open on YouTube"
        />
      {/if}
      {#if onAcknowledgeToggle}
        <button
          type="button"
          class={`inline-flex h-9 items-center gap-2 rounded-full border px-3 text-[11px] font-bold uppercase tracking-[0.08em] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:cursor-not-allowed disabled:opacity-30 ${
            acknowledged
              ? "border-[var(--accent)]/20 bg-[var(--accent-wash-strong)] text-[var(--accent-strong)] shadow-sm"
              : "border-[var(--accent-border-soft)] bg-[var(--panel-surface)] text-[var(--soft-foreground)] hover:border-[var(--accent)]/30 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
          }`}
          data-tooltip={acknowledged ? "Mark as unread" : "Mark as read"}
          data-tooltip-anchor="end"
          data-tooltip-placement="bottom"
          aria-label={acknowledged ? "Mark as unread" : "Mark as read"}
          aria-pressed={acknowledged}
          onclick={onAcknowledgeToggle}
          disabled={busy}
        >
          <span
            class={`flex h-5 w-5 items-center justify-center rounded-full border transition-all ${
              acknowledged
                ? "border-[var(--accent)] bg-[var(--accent)] text-white"
                : "border-[var(--border-soft)] bg-[var(--background)] text-transparent"
            }`}
            aria-hidden="true"
          >
            <svg
              class={`h-3 w-3 transition-opacity ${acknowledged ? "opacity-100" : "opacity-0"}`}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </span>
          <span>{acknowledged ? "Read" : "Unread"}</span>
        </button>
      {/if}
      {#if showEditAction}
        <ContentActionButton
          compact
          icon="edit"
          disabled={busy}
          label="Edit distillation"
          tooltip="Edit distillation"
          tooltipAnchor="end"
          onClick={onEdit}
        />
      {/if}
    </div>
  </div>
{/if}
