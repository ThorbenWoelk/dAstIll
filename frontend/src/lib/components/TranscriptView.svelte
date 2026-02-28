<script lang="ts">
	export let html = '';
	export let formatting = false;
</script>

<article
	class={`prose prose-stone prose-lg max-w-none break-words leading-relaxed transition-colors prose-headings:text-balance prose-headings:font-serif prose-headings:font-normal prose-h1:text-4xl prose-h2:text-3xl prose-h3:text-2xl prose-p:leading-relaxed prose-p:text-[#333] prose-strong:font-semibold prose-a:text-[var(--accent)] prose-a:underline-offset-4 ${
		formatting
			? 'animate-pulse text-[var(--soft-foreground)] prose-headings:text-[var(--soft-foreground)] prose-strong:text-[var(--soft-foreground)]'
			: 'text-[var(--foreground)] prose-headings:text-[var(--foreground)] prose-strong:text-[var(--foreground)]'
	}`}
>
	{@html html}
</article>
