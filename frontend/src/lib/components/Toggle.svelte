<script lang="ts">
	export let options: string[] = [];
	export let value = "";
	export let onChange: (next: string) => void = () => {};
</script>

<div class="inline-flex items-center gap-6 h-10" role="tablist" aria-label="Content Mode">
	{#each options as option}
		<button
			type="button"
			role="tab"
			aria-selected={value === option}
			class={`group flex items-center gap-2 h-full text-[10.5px] font-bold uppercase tracking-[0.2em] transition-all duration-300 focus-visible:outline-none relative ${
				value === option
					? "text-[var(--foreground)]"
					: "text-[var(--soft-foreground)] opacity-50 hover:opacity-100 hover:text-[var(--foreground)]"
			}`}
			onclick={() => onChange(option)}
		>
			{#if option === "transcript"}
				<svg
					width="13"
					height="13"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="transition-transform duration-300 group-hover:-translate-y-0.5"
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
					width="13"
					height="13"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="transition-transform duration-300 group-hover:-translate-y-0.5"
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
					width="13"
					height="13"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="transition-transform duration-300 group-hover:-translate-y-0.5"
				>
					<circle cx="12" cy="12" r="9" />
					<path d="M12 17v-5" />
					<path d="M12 8h.01" />
				</svg>
			{/if}
			<span>{option}</span>
			
			{#if value === option}
				<div class="absolute bottom-0 left-0 right-0 h-[2px] bg-[var(--accent)] rounded-full"></div>
			{/if}
		</button>
	{/each}
</div>
