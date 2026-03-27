<script lang="ts">
  import { tick } from "svelte";

  let {
    open,
    label,
    inputValue,
    saving = false,
    readOnly = false,
    popupStackClass = "flex flex-col-reverse gap-2",
    wrapperClass = "",
    buttonClass = "",
    readonlyClass = "",
    dialogClass = "",
    inputClass = "",
    submitClass = "",
    readonlyId = undefined as string | undefined,
    onToggle,
    onInputValueChange,
    onSubmit,
  }: {
    open: boolean;
    label: string;
    inputValue: string;
    saving?: boolean;
    readOnly?: boolean;
    popupStackClass?: string;
    wrapperClass?: string;
    buttonClass?: string;
    readonlyClass?: string;
    dialogClass?: string;
    inputClass?: string;
    submitClass?: string;
    readonlyId?: string;
    onToggle: () => void;
    onInputValueChange: (value: string) => void;
    onSubmit: () => void;
  } = $props();

  function scrollIntoViewOnMount(node: HTMLElement) {
    void tick().then(() =>
      node.scrollIntoView({ behavior: "smooth", block: "nearest" }),
    );
  }

  function handleInput(event: Event) {
    onInputValueChange((event.currentTarget as HTMLInputElement).value);
  }
</script>

{#if readOnly}
  <p id={readonlyId} class={readonlyClass}>
    Synced to {label}
  </p>
{:else}
  <div class={wrapperClass}>
    <div class={popupStackClass}>
      <button
        type="button"
        class={buttonClass}
        aria-expanded={open}
        aria-haspopup="dialog"
        aria-label="Adjust sync date"
        onclick={onToggle}
      >
        <span class="font-bold uppercase tracking-[0.06em] opacity-70"
          >Synced to</span
        >
        <span
          class="border-b border-dotted border-[var(--border-soft)] text-[var(--foreground)]"
          >{label}</span
        >
      </button>
      {#if open}
        <div
          class={dialogClass}
          role="dialog"
          aria-label="Sync from date"
          use:scrollIntoViewOnMount
        >
          <input
            type="date"
            class={inputClass}
            value={inputValue}
            disabled={saving}
            oninput={handleInput}
          />
          <button
            type="button"
            class={submitClass}
            onclick={onSubmit}
            disabled={!inputValue || saving}
          >
            {saving ? "..." : "Set"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
