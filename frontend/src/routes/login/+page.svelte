<script lang="ts">
  import type { ActionData, PageData } from "./$types";

  let {
    data,
    form,
  }: {
    data: PageData;
    form?: ActionData;
  } = $props();
</script>

<svelte:head>
  <title>Admin sign in — dAstIll</title>
</svelte:head>

<div class="flex min-h-full items-center justify-center px-6 py-16">
  <form
    method="POST"
    class="w-full max-w-sm space-y-6 rounded-[var(--radius-lg)] bg-[var(--panel-surface)] px-6 py-7 shadow-sm"
  >
    <div class="space-y-2">
      <p
        class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
      >
        Admin access
      </p>
      <h1
        class="font-serif text-[28px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        Sign in
      </h1>
      <p class="text-[13px] leading-6 text-[var(--soft-foreground)]">
        Enter the admin password to unlock operator actions.
      </p>
    </div>

    <input
      type="hidden"
      name="redirectTo"
      value={form?.redirectTo ?? data.redirectTo}
    />

    <label class="block space-y-2">
      <span
        class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
      >
        Password
      </span>
      <input
        name="password"
        type="password"
        autocomplete="current-password"
        class="w-full rounded-full bg-[var(--surface)] px-4 py-3 text-[14px] text-[var(--foreground)] outline-none transition focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        required
      />
    </label>

    {#if form?.message}
      <p class="text-[12px] text-[var(--danger)]">{form.message}</p>
    {/if}

    <button
      type="submit"
      class="inline-flex h-11 w-full items-center justify-center rounded-full bg-[var(--foreground)] px-5 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--background)] transition hover:bg-[var(--accent-strong)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
    >
      Continue
    </button>
  </form>
</div>
