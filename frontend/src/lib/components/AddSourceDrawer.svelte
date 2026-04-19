<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import { planOpenAlexQuery } from "$lib/api";
  import {
    buildOpenAlexInterpretationStatus,
    buildEmptyOpenAlexPlan,
    prepareOpenAlexPlanForSubmit,
    syncOpenAlexPlanFromDraft,
    type OpenAlexInterpretationStatus,
  } from "$lib/openalex-plan-state";
  import type {
    OpenAlexSavedSearchQuery,
    OpenAlexSearchScope,
    OpenAlexSort,
  } from "$lib/types";
  import type { AddSourceSubmission } from "$lib/workspace/component-props";
  import { fly } from "svelte/transition";

  type SourceMode = "youtube" | "openalex" | "podcast" | "website";

  let {
    open = false,
    busy = false,
    errorMessage = null as string | null,
    onOpen = () => {},
    onClose,
    onSubmit,
  }: {
    open?: boolean;
    busy?: boolean;
    errorMessage?: string | null;
    onOpen?: () => void;
    onClose: () => void;
    onSubmit: (input: AddSourceSubmission) => Promise<boolean> | boolean;
  } = $props();

  const modeOrder: SourceMode[] = ["youtube", "openalex", "podcast", "website"];
  const modeMeta: Record<
    SourceMode,
    {
      label: string;
      eyebrow: string;
      description: string;
      placeholder: string;
      examples: string[];
      buildInput: (value: string) => string;
      submitLabel: string;
    }
  > = {
    youtube: {
      label: "YouTube",
      eyebrow: "Channel",
      description:
        "Paste a handle, channel URL, or video URL. Video URLs still attach to the right source automatically.",
      placeholder: "@healthyGamerGG or https://youtube.com/@healthyGamerGG",
      examples: ["@healthyGamerGG", "https://youtube.com/@veritasium"],
      buildInput: (value) => value.trim(),
      submitLabel: "Subscribe to YouTube source",
    },
    openalex: {
      label: "OpenAlex",
      eyebrow: "Saved search",
      description:
        "Create a durable publication feed from a query. New matching papers will refresh into the library.",
      placeholder: "recent multimodal ai papers",
      examples: ["recent multimodal ai papers", "protein folding diffusion"],
      buildInput: (value) => `openalex: ${value.trim()}`,
      submitLabel: "Create OpenAlex saved search",
    },
    podcast: {
      label: "Podcast RSS",
      eyebrow: "Feed",
      description:
        "Paste a podcast RSS feed URL. Episodes sync under one source and show notes become readable content when available.",
      placeholder: "https://feeds.simplecast.com/54nAGcIl",
      examples: [
        "https://feeds.simplecast.com/54nAGcIl",
        "https://rss.art19.com/the-daily",
      ],
      buildInput: (value) => `podcast: ${value.trim()}`,
      submitLabel: "Subscribe to podcast feed",
    },
    website: {
      label: "Website page",
      eyebrow: "Tracked page",
      description:
        "Track a single page directly. The page is fetched into the reading flow and can be refreshed later.",
      placeholder: "https://example.com/article",
      examples: [
        "https://example.com/article",
        "site: https://blog.example.com/post",
      ],
      buildInput: (value) => `site: ${value.trim()}`,
      submitLabel: "Track website page",
    },
  };

  let activeMode = $state<SourceMode>("youtube");
  let draftByMode = $state<Record<SourceMode, string>>({
    youtube: "",
    openalex: "",
    podcast: "",
    website: "",
  });
  let inputEl = $state<HTMLInputElement | null>(null);
  let openAlexPlan = $state<OpenAlexSavedSearchQuery | null>(null);
  let planningOpenAlex = $state(false);
  let openAlexPlanError = $state<string | null>(null);
  let openAlexInterpretation = $state<OpenAlexInterpretationStatus | null>(
    null,
  );
  let openAlexInterpretationRequestId = 0;
  const canSubmitOpenAlex = $derived(
    prepareOpenAlexPlanForSubmit(
      openAlexPlan,
      draftByMode.openalex,
    ).query_text.trim().length > 0,
  );

  $effect(() => {
    if (!open) return;
    const handle = requestAnimationFrame(() => inputEl?.focus());
    return () => cancelAnimationFrame(handle);
  });

  function setMode(mode: SourceMode) {
    activeMode = mode;
    openAlexPlanError = null;
    if (mode === "openalex" && !openAlexPlan) {
      openAlexPlan = buildEmptyOpenAlexPlan(draftByMode.openalex);
    }
  }

  function setDraft(value: string) {
    if (activeMode === "openalex") {
      openAlexPlan = syncOpenAlexPlanFromDraft(
        openAlexPlan,
        draftByMode.openalex,
        value,
      );
    }
    draftByMode = {
      ...draftByMode,
      [activeMode]: value,
    };
  }

  function handleDraftInput(event: Event) {
    setDraft((event.currentTarget as HTMLInputElement).value);
  }

  function handleOpenAlexQueryTextInput(event: Event) {
    updateOpenAlexPlan(
      "query_text",
      (event.currentTarget as HTMLInputElement).value,
    );
  }

  function handleOpenAlexFromDateInput(event: Event) {
    updateOpenAlexPlan(
      "from_publication_date",
      (event.currentTarget as HTMLInputElement).value || null,
    );
  }

  function handleOpenAlexToDateInput(event: Event) {
    updateOpenAlexPlan(
      "to_publication_date",
      (event.currentTarget as HTMLInputElement).value || null,
    );
  }

  function handleOpenAlexWorkTypeInput(event: Event) {
    updateOpenAlexPlan(
      "work_type",
      (event.currentTarget as HTMLInputElement).value || null,
    );
  }

  function handleOpenAlexOpenAccessInput(event: Event) {
    updateOpenAlexPlan(
      "open_access_only",
      (event.currentTarget as HTMLInputElement).checked ? true : null,
    );
  }

  function setOpenAlexSearchScope(value: OpenAlexSearchScope) {
    updateOpenAlexPlan("search_scope", value);
  }

  function setOpenAlexSort(value: OpenAlexSort) {
    updateOpenAlexPlan("sort", value);
  }

  function close() {
    onClose();
  }

  function openDrawer() {
    onOpen();
  }

  async function submit() {
    const rawValue = draftByMode[activeMode].trim();
    if (busy) return;
    let payload: AddSourceSubmission;

    if (activeMode === "openalex") {
      const preparedPlan = prepareOpenAlexPlanForSubmit(
        openAlexPlan,
        draftByMode.openalex,
      );
      if (!preparedPlan.query_text.trim()) {
        return;
      }
      payload = {
        input: modeMeta[activeMode].buildInput(
          preparedPlan.natural_language_query,
        ),
        openalex_query: preparedPlan,
      };
    } else {
      if (!rawValue) return;
      payload = modeMeta[activeMode].buildInput(rawValue);
    }

    const success = await onSubmit(payload);
    if (!success) return;
    draftByMode = {
      youtube: "",
      openalex: "",
      podcast: "",
      website: "",
    };
    openAlexPlan = null;
    openAlexPlanError = null;
    close();
  }

  async function interpretOpenAlexQuery() {
    const rawValue = draftByMode.openalex.trim();
    if (!rawValue || planningOpenAlex || busy) return;
    const requestId = ++openAlexInterpretationRequestId;
    planningOpenAlex = true;
    openAlexPlanError = null;
    openAlexInterpretation = buildOpenAlexInterpretationStatus("preparing");
    close();

    try {
      await Promise.resolve();
      if (requestId !== openAlexInterpretationRequestId) {
        return;
      }
      openAlexInterpretation = buildOpenAlexInterpretationStatus("planning");
      const response = await planOpenAlexQuery(rawValue);
      if (requestId !== openAlexInterpretationRequestId) {
        return;
      }
      openAlexPlan = response.query;
      activeMode = "openalex";
      draftByMode = {
        ...draftByMode,
        openalex: response.query.natural_language_query,
      };
      openAlexInterpretation = null;
      openDrawer();
    } catch (error) {
      if (requestId !== openAlexInterpretationRequestId) {
        return;
      }
      openAlexPlanError = (error as Error).message;
      openAlexInterpretation = buildOpenAlexInterpretationStatus("failed");
      activeMode = "openalex";
      openDrawer();
    } finally {
      if (requestId === openAlexInterpretationRequestId) {
        planningOpenAlex = false;
      }
    }
  }

  function updateOpenAlexPlan<K extends keyof OpenAlexSavedSearchQuery>(
    key: K,
    value: OpenAlexSavedSearchQuery[K],
  ) {
    if (!openAlexPlan) return;
    openAlexPlan = {
      ...openAlexPlan,
      [key]: value,
    };
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    await submit();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape") {
      close();
    }
  }

  $effect(() => {
    if (!open || openAlexInterpretation?.phase !== "failed") {
      return;
    }
    openAlexInterpretation = null;
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 flex justify-end"
    style="z-index: var(--z-drawer);"
    role="dialog"
    aria-modal="true"
    aria-labelledby="add-source-title"
  >
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      aria-label="Close add source drawer"
      onclick={close}
    ></button>

    <section
      use:clickOutside={{ enabled: open, onClickOutside: close }}
      class="relative flex h-full w-full max-w-xl flex-col overflow-hidden bg-[var(--surface)] shadow-2xl"
      transition:fly={{ x: 24, duration: 180 }}
    >
      <header class="px-5 pt-5 pb-3">
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <p
              class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
            >
              Add Source
            </p>
            <h2
              id="add-source-title"
              class="mt-1 text-lg font-semibold tracking-tight text-[var(--foreground)]"
            >
              Pick a source type
            </h2>
            <p
              class="mt-1 text-sm leading-relaxed text-[var(--soft-foreground)]"
            >
              Choose the kind of source you want to subscribe to, then review
              the source details that apply.
            </p>
          </div>
          <button
            type="button"
            class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
            onclick={close}
            aria-label="Close add source drawer"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </header>

      <div class="flex gap-4 overflow-x-auto px-5 pb-2 sm:hidden">
        {#each modeOrder as mode}
          <button
            type="button"
            class={`shrink-0 border-b pb-2 text-left transition-colors ${
              activeMode === mode
                ? "border-[var(--accent)] text-[var(--foreground)]"
                : "border-transparent text-[var(--soft-foreground)] opacity-75"
            }`}
            onclick={() => setMode(mode)}
            aria-pressed={activeMode === mode}
          >
            <span
              class="block text-[10px] font-bold uppercase tracking-[0.1em] opacity-65"
            >
              {modeMeta[mode].eyebrow}
            </span>
            <span class="mt-0.5 block text-sm font-semibold">
              {modeMeta[mode].label}
            </span>
          </button>
        {/each}
      </div>

      <div class="hidden gap-5 px-5 pb-2 sm:flex sm:flex-wrap">
        {#each modeOrder as mode}
          <button
            type="button"
            class={`border-b pb-2 text-left transition-colors ${
              activeMode === mode
                ? "border-[var(--accent)] text-[var(--foreground)]"
                : "border-transparent text-[var(--soft-foreground)] opacity-75 hover:opacity-100"
            }`}
            onclick={() => setMode(mode)}
            aria-pressed={activeMode === mode}
          >
            <span
              class="text-[10px] font-bold uppercase tracking-[0.1em] opacity-65"
            >
              {modeMeta[mode].eyebrow}
            </span>
            <span class="ml-2 text-sm font-semibold"
              >{modeMeta[mode].label}</span
            >
          </button>
        {/each}
      </div>

      <form
        class="flex min-h-0 flex-1 flex-col px-5 py-4"
        onsubmit={handleSubmit}
      >
        <div class="min-h-0 flex-1 overflow-y-auto">
          <p
            class="max-w-[34rem] text-sm leading-relaxed text-[var(--soft-foreground)]"
          >
            {activeMode === "openalex"
              ? "Create a durable publication feed from natural language or by setting the filters directly. AI prefill is optional."
              : modeMeta[activeMode].description}
          </p>

          <div class="mt-4">
            <label
              for="drawer-source-input"
              class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
            >
              {activeMode === "openalex"
                ? "Topic or plain-language request"
                : modeMeta[activeMode].label}
            </label>
            <input
              id="drawer-source-input"
              bind:this={inputEl}
              type="text"
              autocomplete="off"
              spellcheck={false}
              class="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] px-4 py-3 text-sm text-[var(--foreground)] placeholder:text-[var(--soft-foreground)] placeholder:opacity-45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30"
              placeholder={modeMeta[activeMode].placeholder}
              value={draftByMode[activeMode]}
              oninput={handleDraftInput}
            />
            {#if activeMode === "openalex"}
              <div class="mt-3 flex items-center justify-between gap-3">
                <p class="text-xs text-[var(--soft-foreground)]">
                  Optional: use AI to prefill the structured filters below, or
                  edit them directly yourself.
                </p>
                <button
                  type="button"
                  class="shrink-0 text-xs font-semibold text-[var(--foreground)] underline-offset-4 transition-colors hover:text-[var(--accent)] hover:underline disabled:opacity-40"
                  onclick={() => void interpretOpenAlexQuery()}
                  disabled={!draftByMode.openalex.trim() ||
                    planningOpenAlex ||
                    busy}
                >
                  {planningOpenAlex ? "Interpreting..." : "Interpret with AI"}
                </button>
              </div>
            {/if}
          </div>

          {#if activeMode === "openalex" && openAlexPlan}
            <div class="mt-5 space-y-4">
              <div>
                <label
                  for="openalex-query-text"
                  class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                >
                  Query text
                </label>
                <input
                  id="openalex-query-text"
                  type="text"
                  class="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] px-4 py-3 text-sm text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30"
                  value={openAlexPlan.query_text}
                  oninput={handleOpenAlexQueryTextInput}
                />
              </div>

              <div class="grid gap-4 sm:grid-cols-2">
                <div>
                  <label
                    for="openalex-from-date"
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                  >
                    From date
                  </label>
                  <input
                    id="openalex-from-date"
                    type="date"
                    class="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] px-4 py-3 text-sm text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30"
                    value={openAlexPlan.from_publication_date ?? ""}
                    oninput={handleOpenAlexFromDateInput}
                  />
                </div>
                <div>
                  <label
                    for="openalex-to-date"
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                  >
                    To date
                  </label>
                  <input
                    id="openalex-to-date"
                    type="date"
                    class="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] px-4 py-3 text-sm text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30"
                    value={openAlexPlan.to_publication_date ?? ""}
                    oninput={handleOpenAlexToDateInput}
                  />
                </div>
              </div>

              <div class="grid gap-4 sm:grid-cols-2">
                <div>
                  <label
                    for="openalex-work-type"
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                  >
                    Work type
                  </label>
                  <input
                    id="openalex-work-type"
                    type="text"
                    class="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] px-4 py-3 text-sm text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30"
                    placeholder="article, preprint, review-article"
                    value={openAlexPlan.work_type ?? ""}
                    oninput={handleOpenAlexWorkTypeInput}
                  />
                </div>
              </div>

              <div class="grid gap-4 sm:grid-cols-2">
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                  >
                    Search scope
                  </p>
                  <div class="mt-2 flex gap-4">
                    <label class="text-sm text-[var(--foreground)]">
                      <input
                        class="mr-2"
                        type="radio"
                        checked={openAlexPlan.search_scope ===
                          "title_and_abstract"}
                        oninput={() =>
                          setOpenAlexSearchScope("title_and_abstract")}
                      />
                      Title + abstract
                    </label>
                    <label class="text-sm text-[var(--foreground)]">
                      <input
                        class="mr-2"
                        type="radio"
                        checked={openAlexPlan.search_scope === "general_search"}
                        oninput={() => setOpenAlexSearchScope("general_search")}
                      />
                      General
                    </label>
                  </div>
                </div>
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
                  >
                    Sort
                  </p>
                  <div class="mt-2 flex gap-4">
                    <label class="text-sm text-[var(--foreground)]">
                      <input
                        class="mr-2"
                        type="radio"
                        checked={openAlexPlan.sort === "publication_date_desc"}
                        oninput={() => setOpenAlexSort("publication_date_desc")}
                      />
                      Newest first
                    </label>
                    <label class="text-sm text-[var(--foreground)]">
                      <input
                        class="mr-2"
                        type="radio"
                        checked={openAlexPlan.sort === "relevance_score_desc"}
                        oninput={() => setOpenAlexSort("relevance_score_desc")}
                      />
                      Relevance
                    </label>
                  </div>
                </div>
              </div>

              <label
                class="flex items-center gap-2 text-sm text-[var(--foreground)]"
              >
                <input
                  type="checkbox"
                  checked={openAlexPlan.open_access_only === true}
                  oninput={handleOpenAlexOpenAccessInput}
                />
                Open access only
              </label>
            </div>
          {/if}

          <div class="mt-4">
            <p
              class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)]"
            >
              Examples
            </p>
            <div class="mt-2 flex flex-wrap gap-x-4 gap-y-2">
              {#each modeMeta[activeMode].examples as example}
                <button
                  type="button"
                  class="text-left text-xs font-medium text-[var(--soft-foreground)] underline-offset-4 transition-colors hover:text-[var(--foreground)] hover:underline"
                  onclick={() => setDraft(example)}
                >
                  {example}
                </button>
              {/each}
            </div>
          </div>

          {#if activeMode === "youtube"}
            <p
              class="mt-4 text-xs leading-relaxed text-[var(--soft-foreground)]"
            >
              You can still paste a single YouTube video URL here. The app will
              attach it to the right source automatically.
            </p>
          {/if}

          {#if activeMode === "openalex" && openAlexPlanError}
            <p class="mt-4 text-sm font-medium text-[var(--danger)]">
              {openAlexPlanError}
            </p>
          {/if}

          {#if errorMessage}
            <p class="mt-4 text-sm font-medium text-[var(--danger)]">
              {errorMessage}
            </p>
          {/if}
        </div>

        <div
          class="mt-5 flex flex-col gap-2 pt-2 pb-[max(0.75rem,env(safe-area-inset-bottom))] sm:flex-row sm:justify-end"
        >
          <button
            type="button"
            class="inline-flex items-center justify-center rounded-[var(--radius-md)] px-4 py-2.5 text-sm font-medium text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)]"
            onclick={close}
          >
            Cancel
          </button>
          <button
            type="submit"
            class="inline-flex items-center justify-center rounded-[var(--radius-md)] bg-[var(--accent)] px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-[var(--accent-strong)] disabled:opacity-50"
            disabled={activeMode === "openalex"
              ? !canSubmitOpenAlex || busy
              : !draftByMode[activeMode].trim() || busy}
          >
            {busy ? "Adding..." : modeMeta[activeMode].submitLabel}
          </button>
        </div>
      </form>
    </section>
  </div>
{/if}

{#if openAlexInterpretation && !open}
  <div
    class="mobile-bottom-stack-offset fixed bottom-6 left-1/2 z-[111] flex w-[min(92vw,28rem)] -translate-x-1/2 items-start gap-3 rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] px-4 py-3 shadow-lg"
    role="status"
    aria-live="polite"
  >
    <div class="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center">
      {#if openAlexInterpretation.phase === "failed"}
        <span
          class="h-2.5 w-2.5 rounded-full bg-[var(--danger)]"
          aria-hidden="true"
        ></span>
      {:else}
        <span
          class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-[var(--accent-border-soft)] border-t-[var(--accent)]"
          aria-hidden="true"
        ></span>
      {/if}
    </div>

    <div class="min-w-0 flex-1">
      <p
        class="text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
      >
        OpenAlex AI
      </p>
      <p class="mt-1 text-[13px] font-semibold text-[var(--foreground)]">
        {openAlexInterpretation.stateLabel}
      </p>
      <p class="mt-1 text-[13px] leading-5 text-[var(--soft-foreground)]">
        {openAlexInterpretation.message}
      </p>
    </div>
  </div>
{/if}
