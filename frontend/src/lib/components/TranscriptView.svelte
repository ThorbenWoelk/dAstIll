<script lang="ts">
	export let html = '';
	export let formatting = false;
</script>

<article
	class={`prose max-w-none break-words leading-relaxed transition-all duration-500 prose-headings:font-serif prose-headings:font-bold prose-headings:tracking-tight prose-h1:text-5xl prose-h2:text-3xl prose-h3:text-2xl prose-p:text-[17px] prose-p:leading-[1.75] prose-p:tracking-[-0.01em] prose-strong:font-bold prose-a:text-[var(--accent)] prose-a:underline-offset-4 prose-blockquote:border-l-[var(--accent)] prose-blockquote:bg-[var(--accent-soft)]/30 prose-blockquote:py-1 prose-blockquote:px-6 prose-blockquote:rounded-r-lg ${
		formatting
			? 'opacity-40 grayscale blur-[1px]'
			: 'opacity-100'
	}`}
>
	{@html html}
</article>

<style>
	:global(.prose h1, .prose h2, .prose h3) {
		font-variation-settings: "opsz" 72, "wght" 700;
	}
</style>
