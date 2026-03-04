<script lang="ts">
	export let value = "";
	export let editing = false;
	export let busy = false;
	export let formatting = false;
	export let reverting = false;
	export let showFormatAction = false;
	export let showRevertAction = false;
	export let canRevert = true;
	export let youtubeUrl: string | null = null;
	export let onEdit: () => void = () => {};
	export let onCancel: () => void = () => {};
	export let onSave: () => void = () => {};
	export let onFormat: () => void = () => {};
	export let onRevert: () => void = () => {};
	export let onChange: (next: string) => void = () => {};
	export let onAcknowledgeToggle: (() => void) | undefined = undefined;
	export let acknowledged = false;
	export let aiAvailable = true;
</script>

{#if editing}
	<div class="flex flex-col gap-4 fade-in">
		<div class="flex flex-wrap items-center gap-3">
			{#if showFormatAction}
				<button
					type="button"
					class="inline-flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--background)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--accent)] hover:shadow-sm disabled:opacity-30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
					onclick={onFormat}
					disabled={busy || formatting || reverting || !aiAvailable}
					aria-label={formatting
						? "Formatting transcript"
						: aiAvailable
							? "Clean formatting"
							: "AI Engine: Offline"}
					title={formatting
						? "Formatting…"
						: aiAvailable
							? "Clean formatting"
							: "AI Engine: Offline"}
				>
					{#if formatting}
						<svg
							viewBox="0 0 24 24"
							class="h-4 w-4 animate-spin"
							aria-hidden="true"
						>
							<circle
								cx="12"
								cy="12"
								r="9"
								fill="none"
								stroke="currentColor"
								stroke-opacity="0.25"
								stroke-width="2"
							/>
							<path
								d="M12 3a9 9 0 0 1 9 9"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
							/>
						</svg>
					{:else}
						<svg
							viewBox="0 0 16 16"
							class="h-4 w-4"
							aria-hidden="true"
						>
							<path
								d="M3 4h10M3 8h6M3 12h8"
								fill="none"
								stroke="currentColor"
								stroke-linecap="round"
								stroke-width="2"
							/>
						</svg>
					{/if}
				</button>
				{#if showRevertAction}
					<button
						type="button"
						class="inline-flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--background)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--accent)] hover:shadow-sm disabled:opacity-30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
						onclick={onRevert}
						disabled={busy || formatting || reverting || !canRevert}
						aria-label={reverting
							? "Reverting transcript"
							: "Revert to original transcript"}
						title={reverting
							? "Reverting…"
							: "Revert to original transcript"}
					>
						{#if reverting}
							<svg
								viewBox="0 0 24 24"
								class="h-4 w-4 animate-spin"
								aria-hidden="true"
							>
								<circle
									cx="12"
									cy="12"
									r="9"
									fill="none"
									stroke="currentColor"
									stroke-opacity="0.25"
									stroke-width="2"
								/>
								<path
									d="M12 3a9 9 0 0 1 9 9"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
								/>
							</svg>
						{:else}
							<svg
								viewBox="0 0 24 24"
								class="h-4 w-4"
								aria-hidden="true"
							>
								<path
									d="M9 15 3 9m0 0 6-6M3 9h10.5a5.5 5.5 0 0 1 0 11H9"
									fill="none"
									stroke="currentColor"
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2.5"
								/>
							</svg>
						{/if}
					</button>
				{/if}
				{#if youtubeUrl}
					<a
						href={youtubeUrl}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--background)] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--accent)] hover:shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
						aria-label="Open video on YouTube"
						title="Open on YouTube"
					>
						<svg
							viewBox="0 0 24 24"
							class="h-4 w-4"
							aria-hidden="true"
						>
							<rect
								x="3"
								y="6"
								width="18"
								height="12"
								rx="2"
								ry="2"
								fill="none"
								stroke="currentColor"
								stroke-width="2.5"
							/>
							<path
								d="M10 9l5 3-5 3V9z"
								fill="currentColor"
							/>
						</svg>
					</a>
				{/if}
			{/if}
			<div class="ml-auto flex items-center gap-3">
				<button
					type="button"
					class="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-6 py-2.5 text-[10px] font-bold uppercase tracking-[0.25em] text-white transition-all hover:bg-[var(--accent-strong)] hover:shadow-lg hover:shadow-[var(--accent-strong)]/20 disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
					onclick={onSave}
					disabled={busy}
				>
					{busy ? "RECORDING…" : "PERSIST"}
				</button>
				<button
					type="button"
					class="rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--surface)] px-6 py-2.5 text-[10px] font-bold uppercase tracking-[0.25em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--muted)]/40 disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
					onclick={onCancel}
					disabled={busy}
				>
					ABORT
				</button>
			</div>
		</div>
		<textarea
			name="content"
			autocomplete="off"
			aria-label="Content editor"
			class={`min-h-[400px] w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] p-8 text-[15px] font-medium leading-[1.7] shadow-sm transition-all focus-within:ring-2 focus-within:ring-[var(--accent)]/10 focus-within:border-[var(--accent)]/40 focus-visible:outline-none ${
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
			<button
				type="button"
				class="inline-flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] text-[var(--soft-foreground)] border border-transparent hover:border-[var(--border-soft)] hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] transition-all disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
				onclick={onFormat}
				disabled={busy || formatting || reverting || !aiAvailable}
				aria-label={formatting
					? "Formatting transcript"
					: aiAvailable
						? "Clean formatting"
						: "AI Engine: Offline"}
				title={formatting
					? "Formatting…"
					: aiAvailable
						? "Clean formatting"
						: "AI Engine: Offline"}
			>
				{#if formatting}
					<svg
						viewBox="0 0 24 24"
						class="h-4 w-4 animate-spin"
						aria-hidden="true"
					>
						<circle
							cx="12"
							cy="12"
							r="9"
							fill="none"
							stroke="currentColor"
							stroke-opacity="0.25"
							stroke-width="2"
						/>
						<path
							d="M12 3a9 9 0 0 1 9 9"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
						/>
					</svg>
				{:else}
					<svg
						viewBox="0 0 16 16"
						class="h-4 w-4"
						aria-hidden="true"
					>
						<path
							d="M3 4h10M3 8h6M3 12h8"
							fill="none"
							stroke="currentColor"
							stroke-linecap="round"
							stroke-width="2.2"
						/>
					</svg>
				{/if}
			</button>
			{#if showRevertAction}
				<button
					type="button"
					class="inline-flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] text-[var(--soft-foreground)] border border-transparent hover:border-[var(--border-soft)] hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] transition-all disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
					onclick={onRevert}
					disabled={busy || formatting || reverting || !canRevert}
					aria-label={reverting
						? "Reverting transcript"
						: "Revert to original transcript"}
					title={reverting
						? "Reverting…"
						: "Revert to original transcript"}
				>
					{#if reverting}
						<svg
							viewBox="0 0 24 24"
							class="h-4 w-4 animate-spin"
							aria-hidden="true"
						>
							<circle
								cx="12"
								cy="12"
								r="9"
								fill="none"
								stroke="currentColor"
								stroke-opacity="0.25"
								stroke-width="2"
							/>
							<path
								d="M12 3a9 9 0 0 1 9 9"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
							/>
						</svg>
					{:else}
						<svg
							viewBox="0 0 24 24"
							class="h-4 w-4"
							aria-hidden="true"
						>
							<path
								d="M9 15 3 9m0 0 6-6M3 9h10.5a5.5 5.5 0 0 1 0 11H9"
								fill="none"
								stroke="currentColor"
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2.2"
							/>
						</svg>
					{/if}
				</button>
			{/if}
			{#if youtubeUrl}
				<a
					href={youtubeUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="inline-flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] text-[var(--soft-foreground)] border border-transparent hover:border-[var(--border-soft)] hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
					aria-label="Open video on YouTube"
					title="Open on YouTube"
				>
					<svg
						viewBox="0 0 24 24"
						class="h-4 w-4"
						aria-hidden="true"
					>
						<rect
							x="3"
							y="6"
							width="18"
							height="12"
							rx="2"
							ry="2"
							fill="none"
							stroke="currentColor"
							stroke-width="2.2"
						/>
						<path
							d="M10 9l5 3-5 3V9z"
							fill="currentColor"
						/>
					</svg>
				</a>
			{/if}
		{/if}
		<button
			type="button"
			class="inline-flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] text-[var(--soft-foreground)] border border-transparent hover:border-[var(--border-soft)] hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:opacity-20"
			onclick={onEdit}
			disabled={busy}
			title="Edit distillation"
			aria-label="Edit distillation"
		>
			<svg
				width="15"
				height="15"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				><path d="M12 20h9" /><path
					d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"
				/></svg
			>
		</button>
		{#if onAcknowledgeToggle}
			<div class="h-4 w-px bg-[var(--border-soft)] mx-1"></div>
			<label
				class="flex items-center justify-center h-9 w-9 cursor-pointer group transition-opacity hover:opacity-100"
				title={acknowledged ? "Mark as unread" : "Mark as read"}
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

