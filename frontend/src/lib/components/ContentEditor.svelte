<script lang="ts">
  import ContentActionButton from "./ContentActionButton.svelte";

  export let value = "";
  export let editing = false;
  export let busy = false;
  export let formatting = false;
  export let regenerating = false;
  export let reverting = false;
  export let showFormatAction = false;
  export let showRegenerateAction = false;
  export let showRevertAction = false;
  export let canRevert = true;
  export let youtubeUrl: string | null = null;
  export let onEdit: () => void = () => {};
  export let onCancel: () => void = () => {};
  export let onSave: () => void = () => {};
  export let onFormat: () => void = () => {};
  export let onRegenerate: () => void = () => {};
  export let onRevert: () => void = () => {};
  export let onChange: (next: string) => void = () => {};
  export let onAcknowledgeToggle: (() => void) | undefined = undefined;
  export let acknowledged = false;
  export let aiAvailable = true;

  $: formatActionLabel = formatting
    ? "Formatting transcript"
    : aiAvailable
      ? "Clean formatting"
      : "auto-format (AI engine required)";
  $: revertActionLabel = reverting
    ? "Reverting transcript"
    : "Revert to original transcript";
  $: regenerateActionLabel = regenerating
    ? "Regenerating summary"
    : aiAvailable
      ? "Regenerate summary"
      : "regenerate (AI engine required)";
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
          : "bg-white"
      }`}
      {value}
      oninput={(event) =>
        onChange((event.currentTarget as HTMLTextAreaElement).value)}
      placeholder="Refine the distillation here…"
    ></textarea>
  </div>
{:else}
  <div class="flex flex-wrap gap-4 items-center">
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
    {#if youtubeUrl}
      <ContentActionButton
        compact
        icon="youtube"
        href={youtubeUrl}
        label="Open video on YouTube"
        tooltip="Open on YouTube"
      />
    {/if}
    <ContentActionButton
      compact
      icon="edit"
      disabled={busy}
      label="Edit distillation"
      tooltip="Edit distillation"
      tooltipAnchor="end"
      onClick={onEdit}
    />
    {#if onAcknowledgeToggle}
      <div class="h-4 w-px bg-[var(--border-soft)] mx-1"></div>
      <label
        class="flex items-center justify-center h-9 w-9 cursor-pointer group transition-opacity hover:opacity-100"
        data-tooltip={acknowledged ? "Mark as unread" : "Mark as read"}
        data-tooltip-anchor="end"
      >
        <div class="relative flex items-center justify-center">
          <input
            type="checkbox"
            class="peer h-5 w-5 cursor-pointer appearance-none rounded-[var(--radius-sm)] border-2 border-[var(--border)] bg-[var(--background)] transition-all checked:border-[var(--accent)] checked:bg-[var(--accent)] hover:border-[var(--accent)]/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:cursor-not-allowed disabled:opacity-30"
            checked={acknowledged}
            onchange={onAcknowledgeToggle}
            disabled={busy}
            aria-label="Toggle read status"
          />
          <svg
            class="absolute h-3.5 w-3.5 text-white opacity-0 transition-opacity peer-checked:opacity-100"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="4"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
      </label>
    {/if}
  </div>
{/if}
