import DOMPurify from 'isomorphic-dompurify';
import { marked } from 'marked';

type RenderMarkdownOptions = {
	preserveLineBreaks?: boolean;
};

function normalizeLineBreaks(input: string) {
	return input.replace(/\r\n?/g, '\n');
}

function hardenSingleLineBreaks(input: string) {
	const lines = input.split('\n');
	let output = '';

	for (let i = 0; i < lines.length; i += 1) {
		const current = lines[i];
		const next = lines[i + 1];

		output += current;

		if (i === lines.length - 1) continue;
		if (current.trim() === '' || (next ?? '').trim() === '') {
			output += '\n';
		} else {
			// Force markdown hard-break behavior for single newlines.
			output += '  \n';
		}
	}

	return output;
}

function isMarkdownStructuralLine(line: string) {
	const trimmed = line.trimStart();
	return (
		trimmed.startsWith('#') ||
		trimmed.startsWith('>') ||
		trimmed.startsWith('- ') ||
		trimmed.startsWith('* ') ||
		trimmed.startsWith('+ ') ||
		/^\d+[.)]\s/.test(trimmed) ||
		trimmed.startsWith('|') ||
		trimmed.startsWith('```')
	);
}

/**
 * Transcript sources often contain hard-wrapped lines.
 * Collapse single wrapped lines into paragraph flow while preserving markdown structure.
 */
export function normalizeTranscriptForRender(input: string) {
	const lines = normalizeLineBreaks(input || '').split('\n');
	const output: string[] = [];
	let inFence = false;

	for (const rawLine of lines) {
		const line = rawLine.trimEnd();
		const trimmed = line.trim();

		if (trimmed.startsWith('```')) {
			inFence = !inFence;
			output.push(line);
			continue;
		}

		if (inFence) {
			output.push(line);
			continue;
		}

		if (trimmed === '') {
			if (output.length === 0 || output[output.length - 1] !== '') {
				output.push('');
			}
			continue;
		}

		if (output.length === 0) {
			output.push(line);
			continue;
		}

		const prev = output[output.length - 1];
		if (prev === '') {
			output.push(line);
			continue;
		}

		if (isMarkdownStructuralLine(prev) || isMarkdownStructuralLine(line)) {
			output.push(line);
			continue;
		}

		output[output.length - 1] = `${prev} ${trimmed}`.replace(/\s+/g, ' ');
	}

	return output.join('\n');
}

export function renderMarkdown(input: string, options: RenderMarkdownOptions = {}) {
	const normalized = normalizeLineBreaks(input || '');
	const prepared = options.preserveLineBreaks === false ? normalized : hardenSingleLineBreaks(normalized);
	const html = marked.parse(prepared, {
		gfm: true,
		breaks: false
	});
	return DOMPurify.sanitize(typeof html === 'string' ? html : '');
}
