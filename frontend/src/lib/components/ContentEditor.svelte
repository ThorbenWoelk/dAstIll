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
</script>

{#if editing}
	<div class="flex flex-col gap-3">
		<div class="flex flex-wrap gap-2">
			{#if showFormatAction}
				<button
					type="button"
					class="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--border)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
					onclick={onFormat}
					disabled={busy || formatting || reverting}
					aria-label={formatting
						? "Formatting transcript"
						: "Clean formatting"}
					title={formatting ? "Formatting…" : "Clean formatting"}
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
								stroke-width="1.4"
							/>
						</svg>
					{/if}
				</button>
				{#if showRevertAction}
					<button
						type="button"
						class="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--border)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
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
								viewBox="0 0 16 16"
								class="h-4 w-4"
								aria-hidden="true"
							>
								<path
									d="M5 4.5H2.5V7M2.5 7A5.5 5.5 0 1 0 4.6 2.7"
									fill="none"
									stroke="currentColor"
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="1.4"
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
						class="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--border)] text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
						aria-label="Open video on YouTube"
						title="Open on YouTube"
					>
						<svg
							viewBox="0 0 24 24"
							class="h-4 w-4"
							aria-hidden="true"
						>
							<rect
								x="3.5"
								y="6.5"
								width="17"
								height="11"
								rx="3"
								ry="3"
								fill="none"
								stroke="currentColor"
								stroke-width="1.8"
							/>
							<path
								d="M11 9.5v5l4-2.5-4-2.5Z"
								fill="currentColor"
							/>
						</svg>
					</a>
				{/if}
			{/if}
			<button
				type="button"
				class="rounded-full bg-[var(--accent)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.2em] text-white transition-colors hover:bg-[var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
				onclick={onSave}
				disabled={busy}
			>
				{busy ? "Saving…" : "Save"}
			</button>
			<button
				type="button"
				class="rounded-full border border-[var(--border)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.2em] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
				onclick={onCancel}
				disabled={busy}
			>
				Cancel
			</button>
		</div>
		<textarea
			name="content"
			autocomplete="off"
			aria-label="Content editor"
			class={`min-h-[240px] w-full rounded-2xl border border-[var(--border)] p-4 text-sm leading-relaxed shadow-soft transition-shadow focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)] ${
				formatting
					? "animate-pulse bg-[var(--muted)]/45 text-[var(--soft-foreground)]"
					: "bg-white/80"
			}`}
			{value}
			oninput={(event) =>
				onChange((event.currentTarget as HTMLTextAreaElement).value)}
			placeholder="Refine the content here…"
		></textarea>
	</div>
{:else}
	<div class="flex flex-wrap gap-2 items-center">
		{#if showFormatAction}
			<button
				type="button"
				class="inline-flex h-8 w-8 items-center justify-center rounded-sm text-[var(--soft-foreground)] hover:bg-[var(--muted)]/40 hover:text-[var(--foreground)] transition-colors disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]"
				onclick={onFormat}
				disabled={busy || formatting || reverting}
				aria-label={formatting
					? "Formatting transcript"
					: "Clean formatting"}
				title={formatting ? "Formatting…" : "Clean formatting"}
			>
				{#if formatting}
					<svg
						viewBox="0 0 24 24"
						class="h-[15px] w-[15px] animate-spin"
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
						class="h-[15px] w-[15px]"
						aria-hidden="true"
					>
						<path
							d="M3 4h10M3 8h6M3 12h8"
							fill="none"
							stroke="currentColor"
							stroke-linecap="round"
							stroke-width="1.8"
						/>
					</svg>
				{/if}
			</button>
			{#if showRevertAction}
				<button
					type="button"
					class="inline-flex h-8 w-8 items-center justify-center rounded-sm text-[var(--soft-foreground)] hover:bg-[var(--muted)]/40 hover:text-[var(--foreground)] transition-colors disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]"
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
							class="h-[15px] w-[15px] animate-spin"
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
							class="h-[15px] w-[15px]"
							aria-hidden="true"
						>
							<path
								d="M5 4.5H2.5V7M2.5 7A5.5 5.5 0 1 0 4.6 2.7"
								fill="none"
								stroke="currentColor"
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="1.8"
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
					class="inline-flex h-8 w-8 items-center justify-center rounded-sm text-[var(--soft-foreground)] hover:bg-[var(--muted)]/40 hover:text-[var(--foreground)] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]"
					aria-label="Open video on YouTube"
					title="Open on YouTube"
				>
					<svg
						viewBox="0 0 24 24"
						class="h-[15px] w-[15px]"
						aria-hidden="true"
					>
						<rect
							x="3.5"
							y="6.5"
							width="17"
							height="11"
							rx="3"
							ry="3"
							fill="none"
							stroke="currentColor"
							stroke-width="1.8"
						/>
						<path
							d="M11 9.5v5l4-2.5-4-2.5Z"
							fill="currentColor"
						/>
					</svg>
				</a>
			{/if}
		{/if}
		<button
			type="button"
			class="inline-flex h-8 w-8 items-center justify-center rounded-sm text-[var(--soft-foreground)] hover:bg-[var(--muted)]/40 hover:text-[var(--foreground)] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60"
			onclick={onEdit}
			disabled={busy}
			title="Edit"
			aria-label="Edit"
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
			<label
				class="flex items-center ml-2 cursor-pointer transition-opacity hover:opacity-80"
				title="Toggle Acknowledged"
			>
				<input
					type="checkbox"
					class="w-4 h-4 rounded border-[var(--border)] text-[var(--accent)] focus:ring-[var(--accent)] accent-[var(--accent)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
					checked={acknowledged}
					onchange={onAcknowledgeToggle}
					disabled={busy}
					aria-label="Toggle Acknowledged"
				/>
			</label>
		{/if}
	</div>
{/if}
