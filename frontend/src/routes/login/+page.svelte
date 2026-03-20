<script lang="ts">
  import type { ActionData, PageData } from "./$types";

  let {
    data,
    form = undefined,
  }: {
    data: PageData;
    form?: ActionData;
  } = $props();
</script>

<svelte:head>
  <title>Login - dAstIll</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center px-6 py-12">
  <div
    class="w-full max-w-md rounded-[20px] bg-[var(--surface)] p-8 shadow-[var(--shadow-soft)]"
  >
    <div class="space-y-3">
      <p
        class="text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
      >
        Operator Access
      </p>
      <h1
        class="font-[Fraunces] text-3xl font-semibold text-[var(--foreground)]"
      >
        Sign in to dAstIll
      </h1>
      <p class="text-sm leading-6 text-[var(--soft-foreground)]">
        This workspace is restricted. Enter the configured app password to
        continue.
      </p>
    </div>

    <form method="POST" class="mt-8 space-y-4">
      <input type="hidden" name="next" value={form?.next ?? data.next} />

      <label class="block space-y-2">
        <span
          class="text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
        >
          Password
        </span>
        <input
          name="password"
          type="password"
          autocomplete="current-password"
          class="w-full rounded-full bg-[var(--background)] px-4 py-3 text-sm text-[var(--foreground)] outline-none transition focus:ring-2 focus:ring-[color:var(--accent)]/40"
          placeholder="Enter app password"
          required
        />
      </label>

      {#if form?.invalidPassword}
        <p class="text-sm text-[var(--danger)]">
          The password was not accepted.
        </p>
      {/if}

      <button
        type="submit"
        class="w-full rounded-full bg-[var(--accent)] px-4 py-3 text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--surface)] transition hover:opacity-90"
      >
        Continue
      </button>
    </form>
  </div>
</div>
