<script lang="ts">
  import { authState } from "$lib/auth/state.svelte";
  import {
    createWebsiteFolder,
    deleteWebsiteFolder,
    reorderWebsiteFolders,
    updateWebsiteFolder,
  } from "$lib/api";
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import type {
    ContentSourceKind,
    LibraryBootstrap,
    LibrarySectionKind,
    LibrarySectionSummary,
    SourceBackingKind,
    SubscriptionContainerKind,
    WebsiteFolder,
  } from "$lib/types";

  let {
    library = null as LibraryBootstrap | null,
  }: {
    library?: LibraryBootstrap | null;
  } = $props();

  let websiteFolders = $derived<WebsiteFolder[]>(
    library?.website_folders ?? [],
  );
  let createName = $state("");
  let creating = $state(false);
  let editingFolderId = $state<string | null>(null);
  let editingName = $state("");
  let busyFolderId = $state<string | null>(null);
  let localError = $state<string | null>(null);

  let sections = $derived(library?.sections ?? []);
  let canManageFolders = $derived(
    authState.current.authState === "authenticated",
  );
  let selectedSection = $derived<LibrarySectionKind | null>(
    library?.selected_source
      ? sectionForSourceKind(library.selected_source.source_kind)
      : null,
  );

  function sectionForSourceKind(kind: ContentSourceKind): LibrarySectionKind {
    switch (kind) {
      case "you_tube_channel":
        return "video_channels";
      case "podcast_series":
        return "podcasts";
      case "publication_series":
      case "saved_search":
      case "authenticated_publisher_source":
        return "publications";
      case "website":
      case "standalone_tracked_source":
        return "websites";
    }
  }

  function formatContainerKinds(kinds: SubscriptionContainerKind[]) {
    return kinds
      .map((kind) => {
        switch (kind) {
          case "series":
            return "Series";
          case "saved_search":
            return "Saved search";
          case "folder":
            return "Folder";
          case "standalone_tracked_source":
            return "Standalone";
        }
      })
      .join(" / ");
  }

  function formatBackingKinds(kinds: SourceBackingKind[]) {
    return kinds
      .map((kind) => {
        switch (kind) {
          case "feed":
            return "Feed-backed";
          case "query":
            return "Query-backed";
          case "manual":
            return "Manually curated";
          case "authenticated":
            return "Account-backed";
        }
      })
      .join(" · ");
  }

  function beginRename(folder: WebsiteFolder) {
    editingFolderId = folder.id;
    editingName = folder.name;
    localError = null;
  }

  function cancelRename() {
    editingFolderId = null;
    editingName = "";
  }

  async function handleCreateFolder() {
    const name = createName.trim();
    if (!name || !canManageFolders || creating) {
      return;
    }

    creating = true;
    localError = null;
    try {
      const folder = await createWebsiteFolder(name);
      websiteFolders = [...websiteFolders, folder].sort(
        (left, right) => left.position - right.position,
      );
      createName = "";
    } catch (error) {
      localError =
        error instanceof Error ? error.message : "Could not create folder.";
    } finally {
      creating = false;
    }
  }

  async function handleRenameFolder(folderId: string) {
    const name = editingName.trim();
    if (!name || !canManageFolders || busyFolderId) {
      return;
    }

    busyFolderId = folderId;
    localError = null;
    try {
      const updated = await updateWebsiteFolder(folderId, name);
      websiteFolders = websiteFolders.map((folder) =>
        folder.id === folderId ? updated : folder,
      );
      cancelRename();
    } catch (error) {
      localError =
        error instanceof Error ? error.message : "Could not rename folder.";
    } finally {
      busyFolderId = null;
    }
  }

  async function handleDeleteFolder(folderId: string) {
    if (!canManageFolders || busyFolderId) {
      return;
    }

    busyFolderId = folderId;
    localError = null;
    try {
      await deleteWebsiteFolder(folderId);
      websiteFolders = websiteFolders
        .filter((folder) => folder.id !== folderId)
        .map((folder, index) => ({ ...folder, position: index }));
      if (editingFolderId === folderId) {
        cancelRename();
      }
    } catch (error) {
      localError =
        error instanceof Error ? error.message : "Could not delete folder.";
    } finally {
      busyFolderId = null;
    }
  }

  async function moveFolder(folderId: string, delta: -1 | 1) {
    const currentIndex = websiteFolders.findIndex(
      (folder) => folder.id === folderId,
    );
    const nextIndex = currentIndex + delta;
    if (
      !canManageFolders ||
      busyFolderId ||
      currentIndex < 0 ||
      nextIndex < 0 ||
      nextIndex >= websiteFolders.length
    ) {
      return;
    }

    const nextOrder = [...websiteFolders];
    const [moved] = nextOrder.splice(currentIndex, 1);
    nextOrder.splice(nextIndex, 0, moved);
    const nextIds = nextOrder.map((folder) => folder.id);

    busyFolderId = folderId;
    localError = null;
    try {
      websiteFolders = await reorderWebsiteFolders(nextIds);
    } catch (error) {
      localError =
        error instanceof Error ? error.message : "Could not reorder folders.";
    } finally {
      busyFolderId = null;
    }
  }
</script>

{#if library}
  <section
    class="shrink-0 border-b border-[var(--border-soft)] px-4 pb-4 pt-4"
    aria-label="Library groups"
  >
    <div class="space-y-4">
      <div class="space-y-1">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
        >
          Library
        </p>
        <h2
          class="font-[var(--font-body,inherit)] text-sm font-semibold text-[var(--foreground)]"
        >
          Sources grouped by type
        </h2>
        <p class="text-sm leading-5 text-[var(--soft-foreground)]">
          Channels, publications, and websites now share one neutral library
          model.
        </p>
      </div>

      <div class="space-y-2">
        {#each sections as section (section.kind)}
          <div
            class={`space-y-1 border-b border-[var(--border-soft)] pb-2 last:border-b-0 last:pb-0 ${selectedSection === section.kind ? "text-[var(--foreground)]" : ""}`}
          >
            <div class="flex items-center justify-between gap-3">
              <p class="min-w-0 text-sm font-semibold text-[var(--foreground)]">
                {section.title}
              </p>
              <p
                class="shrink-0 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
              >
                {section.source_count} sources
              </p>
            </div>
            <p class="text-xs text-[var(--soft-foreground)]">
              {formatContainerKinds(section.container_kinds)}
            </p>
            <p
              class="text-[10px] font-bold uppercase tracking-[0.06em] text-[var(--soft-foreground)]"
            >
              {formatBackingKinds(section.backing_kinds)}
            </p>
          </div>
        {/each}
      </div>

      <div class="space-y-3">
        <div class="space-y-1">
          <div class="flex items-center justify-between gap-3">
            <p
              class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
            >
              Websites
            </p>
            <p
              class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
            >
              {websiteFolders.length} folders
            </p>
          </div>
          <p class="text-sm leading-5 text-[var(--soft-foreground)]">
            Manual folders keep unrelated tracked sites out of the main source
            lists.
          </p>
        </div>

        {#if canManageFolders}
          <div class="flex items-center gap-2">
            <input
              class="min-w-0 flex-1 rounded-full border border-[var(--border-soft)] bg-[var(--surface)] px-3 py-2 text-sm text-[var(--foreground)] outline-none transition-colors placeholder:text-[var(--soft-foreground)] focus:border-[var(--accent)]"
              type="text"
              placeholder="Create website folder"
              bind:value={createName}
              onkeydown={(event) => {
                if (event.key === "Enter") {
                  void handleCreateFolder();
                }
              }}
              disabled={creating}
            />
            <button
              class="rounded-full border border-[var(--border-soft)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
              type="button"
              onclick={() => void handleCreateFolder()}
              disabled={creating || !createName.trim()}
            >
              Add
            </button>
          </div>
        {:else}
          <p class="text-xs text-[var(--soft-foreground)]">
            Sign in to create and manage website folders.
          </p>
        {/if}

        {#if localError}
          <p class="text-xs text-[var(--danger)]">{localError}</p>
        {/if}

        {#if websiteFolders.length === 0}
          <p class="text-sm text-[var(--soft-foreground)]">
            No website folders yet.
          </p>
        {:else}
          <div class="space-y-2">
            {#each websiteFolders as folder, index (folder.id)}
              <div
                class="flex items-start gap-2 border-b border-[var(--border-soft)] pb-2 last:border-b-0 last:pb-0"
              >
                <div class="min-w-0 flex-1">
                  {#if editingFolderId === folder.id}
                    <div class="space-y-2">
                      <input
                        class="w-full rounded-full border border-[var(--border-soft)] bg-[var(--surface)] px-3 py-2 text-sm text-[var(--foreground)] outline-none transition-colors focus:border-[var(--accent)]"
                        type="text"
                        bind:value={editingName}
                        onkeydown={(event) => {
                          if (event.key === "Enter") {
                            void handleRenameFolder(folder.id);
                          } else if (event.key === "Escape") {
                            cancelRename();
                          }
                        }}
                        disabled={busyFolderId === folder.id}
                      />
                      <div class="flex items-center gap-2">
                        <button
                          class="rounded-full border border-[var(--border-soft)] px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
                          type="button"
                          onclick={() => void handleRenameFolder(folder.id)}
                        >
                          Save
                        </button>
                        <button
                          class="rounded-full border border-[var(--border-soft)] px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)]"
                          type="button"
                          onclick={cancelRename}
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  {:else}
                    <div class="space-y-1">
                      <div class="flex items-center justify-between gap-2">
                        <p
                          class="truncate text-sm font-semibold text-[var(--foreground)]"
                        >
                          {folder.name}
                        </p>
                        <p
                          class="shrink-0 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
                        >
                          {index + 1}
                        </p>
                      </div>
                      <p class="text-xs text-[var(--soft-foreground)]">
                        Folder container for manually tracked sites.
                      </p>
                    </div>
                  {/if}
                </div>

                {#if editingFolderId !== folder.id}
                  <div class="flex shrink-0 items-center gap-1">
                    <button
                      class="rounded-full border border-[var(--border-soft)] px-2 py-1 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)] disabled:opacity-40"
                      type="button"
                      onclick={() => void moveFolder(folder.id, -1)}
                      disabled={busyFolderId === folder.id || index === 0}
                    >
                      Up
                    </button>
                    <button
                      class="rounded-full border border-[var(--border-soft)] px-2 py-1 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)] disabled:opacity-40"
                      type="button"
                      onclick={() => void moveFolder(folder.id, 1)}
                      disabled={busyFolderId === folder.id ||
                        index === websiteFolders.length - 1}
                    >
                      Down
                    </button>
                    <button
                      class="rounded-full border border-[var(--border-soft)] px-2 py-1 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)]"
                      type="button"
                      onclick={() => beginRename(folder)}
                      disabled={busyFolderId === folder.id}
                    >
                      Rename
                    </button>
                    <button
                      class="inline-flex h-7 w-7 items-center justify-center rounded-full border border-[var(--border-soft)] text-[var(--soft-foreground)] transition-colors hover:border-[var(--danger)] hover:text-[var(--danger)] disabled:opacity-40"
                      type="button"
                      aria-label={`Delete ${folder.name}`}
                      onclick={() => void handleDeleteFolder(folder.id)}
                      disabled={busyFolderId === folder.id}
                    >
                      <TrashIcon size={14} />
                    </button>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </section>
{/if}
