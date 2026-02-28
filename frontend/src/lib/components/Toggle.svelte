<script lang="ts">
	export let options: string[] = [];
	export let value = "";
	export let onChange: (next: string) => void = () => {};
</script>

<div class="inline-flex gap-4" role="tablist" aria-label="Content Mode">
	{#each options as option}
		<button
			type="button"
			role="tab"
			aria-selected={value === option}
			class={`flex items-center gap-2 pb-1.5 text-[11px] font-semibold uppercase tracking-[0.15em] transition-colors focus-visible:outline-none border-b-[2px] ${
				value === option
					? "border-[var(--accent)] text-[var(--foreground)]"
					: "border-transparent text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
			}`}
			onclick={() => onChange(option)}
		>
			{#if option === "transcript"}
				<svg
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					><path
						d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"
					/><polyline points="14 2 14 8 20 8" /><line
						x1="16"
						y1="13"
						x2="8"
						y2="13"
					/><line x1="16" y1="17" x2="8" y2="17" /><line
						x1="10"
						y1="9"
						x2="8"
						y2="9"
					/></svg
				>
			{:else if option === "summary"}
				<svg
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					><line x1="21" y1="4" x2="3" y2="4" /><line
						x1="14"
						y1="8"
						x2="3"
						y2="8"
					/><line x1="21" y1="12" x2="3" y2="12" /><line
						x1="14"
						y1="16"
						x2="3"
						y2="16"
					/><line x1="21" y1="20" x2="3" y2="20" /></svg
				>
			{:else}
				<svg
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="12" cy="12" r="9" />
					<path d="M12 17v-5" />
					<path d="M12 8h.01" />
				</svg>
			{/if}
			<span>{option}</span>
		</button>
	{/each}
</div>
