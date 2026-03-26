use aws_sdk_polly::{
    Client as PollyClient,
    types::{OutputFormat as PollyOutputFormat, TextType as PollyTextType},
};
use thiserror::Error;

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn replace_markdown_links(input: &str) -> String {
    // Replace:
    // - [label](url) -> label
    // - ![alt](url) -> alt
    let mut out = String::with_capacity(input.len());
    let mut idx = 0usize;

    while idx < input.len() {
        let rest = &input[idx..];
        let next_img = rest.find("![");
        let next_link = rest.find('[');

        let next = match (next_img, next_link) {
            (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let Some(rel_start) = next else {
            out.push_str(&input[idx..]);
            break;
        };

        let start = idx + rel_start;
        out.push_str(&input[idx..start]);

        let after_start = &input[start..];
        if after_start.starts_with("![") {
            // image: ![alt](url)
            let label_start = start + 2; // skip ![
            let label_end = input[label_start..].find(']').map(|i| label_start + i);
            let Some(label_end) = label_end else {
                out.push('!');
                idx = start + 1;
                continue;
            };

            let after_bracket = &input[label_end + 1..];
            let Some(open_paren_rel) = after_bracket.find('(') else {
                out.push_str(&input[start..label_end + 1]);
                idx = label_end + 1;
                continue;
            };
            let open_paren = label_end + 1 + open_paren_rel;

            let after_paren = &input[open_paren + 1..];
            let Some(close_paren_rel) = after_paren.find(')') else {
                out.push_str(&input[start..open_paren]);
                idx = open_paren;
                continue;
            };
            let close_paren = open_paren + 1 + close_paren_rel;

            out.push_str(input[label_start..label_end].trim());
            idx = close_paren + 1;
            continue;
        }

        // normal link: [label](url)
        let label_start = start + 1; // skip [
        let label_end = input[label_start..].find(']').map(|i| label_start + i);
        let Some(label_end) = label_end else {
            out.push('[');
            idx = start + 1;
            continue;
        };

        let after_bracket = &input[label_end + 1..];
        let Some(open_paren_rel) = after_bracket.find('(') else {
            out.push_str(&input[start..label_end + 1]);
            idx = label_end + 1;
            continue;
        };
        let open_paren = label_end + 1 + open_paren_rel;

        let after_paren = &input[open_paren + 1..];
        let Some(close_paren_rel) = after_paren.find(')') else {
            out.push_str(&input[start..open_paren]);
            idx = open_paren;
            continue;
        };
        let close_paren = open_paren + 1 + close_paren_rel;

        out.push_str(input[label_start..label_end].trim());
        idx = close_paren + 1;
    }

    out
}

pub(crate) fn sanitize_markdown_for_tts(input: &str) -> String {
    const BREAK_AFTER_HEADING: &str = r#"<break time="0.6s" />"#;
    const BREAK_AFTER_LIST_ITEM: &str = r#"<break time="0.15s" />"#;

    let no_html = strip_html_tags(input);
    let links_stripped = replace_markdown_links(&no_html);

    let mut out = String::with_capacity(links_stripped.len());
    let mut in_fence = false;

    for line in links_stripped.lines() {
        let line_trim_start = line.trim_start();

        let trimmed = line.trim_start();

        // Toggle code fences, but keep the internal content (we just remove the markers).
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }

        let is_heading = line_trim_start.starts_with('#');
        let mut is_list_item = false;
        let mut is_ordered_list_item = false;

        let mut processed = line.to_string();

        // Headings: remove leading '#' and whitespace.
        let ts = processed.trim_start();
        if ts.starts_with('#') {
            processed = ts.trim_start_matches('#').trim_start().to_string();
        }

        // Blockquotes: remove leading '>' (and one following space).
        let pst = processed.trim_start();
        if let Some(stripped) = pst.strip_prefix('>') {
            processed = stripped.trim_start().to_string();
        }

        // List prefixes.
        let pst2 = processed.trim_start();
        for prefix in ["- ", "* ", "+ "] {
            if let Some(stripped) = pst2.strip_prefix(prefix) {
                processed = stripped.to_string();
                is_list_item = true;
                break;
            }
        }

        // Ordered list prefixes: "1. " / "1) "
        let pst3 = processed.trim_start();
        if pst3.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let mut byte_idx = 0usize;
            for ch in pst3.chars() {
                if ch.is_ascii_digit() {
                    byte_idx += ch.len_utf8();
                } else {
                    break;
                }
            }
            let rest = &pst3[byte_idx..];
            if let Some(after) = rest.strip_prefix(". ") {
                processed = after.to_string();
                is_ordered_list_item = true;
            } else if let Some(after) = rest.strip_prefix(") ") {
                processed = after.to_string();
                is_ordered_list_item = true;
            }
        }

        if processed.is_empty() {
            continue;
        }

        // Remove inline emphasis/code/decorators and table separators.
        let decor_stripped = processed
            .chars()
            .filter(|ch| !matches!(*ch, '*' | '_' | '`' | '~' | '|' | '<' | '>'))
            .collect::<String>();

        let mut cleaned = decor_stripped.trim().to_string();
        // SSML is XML under the hood; escape `&` so it doesn't break parsing.
        // (We already strip `<` and `>` above to avoid untrusted tags.)
        cleaned = cleaned.replace('&', "&amp;");
        if !cleaned.is_empty() {
            let ends_with_punctuation = cleaned
                .chars()
                .last()
                .is_some_and(|c| matches!(c, '.' | '!' | '?'));

            if is_heading {
                if !ends_with_punctuation {
                    cleaned.push('.');
                }
                cleaned.push(' ');
                cleaned.push_str(BREAK_AFTER_HEADING);
            } else if is_list_item || is_ordered_list_item {
                if !ends_with_punctuation {
                    cleaned.push('.');
                }
                cleaned.push(' ');
                cleaned.push_str(BREAK_AFTER_LIST_ITEM);
            }
        }

        if cleaned.is_empty() {
            continue;
        }

        // Add a space between lines rather than keeping raw newlines.
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&cleaned);

        // `in_fence` intentionally isn't used beyond fence marker skipping.
        let _ = in_fence;
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug)]
pub struct PollyTtsService {
    client: PollyClient,
    voice_id: String,
    engine: String,
    output_format: String,
    sample_rate: String,
}

#[derive(Debug, Error)]
pub enum PollyTtsError {
    #[error("summary text is empty")]
    EmptyText,
    #[error("failed to call Amazon Polly API: {0}")]
    Request(String),
}

impl PollyTtsService {
    pub fn new(
        client: PollyClient,
        voice_id: String,
        engine: String,
        output_format: String,
        sample_rate: String,
    ) -> Self {
        Self {
            client,
            voice_id,
            engine,
            output_format,
            sample_rate,
        }
    }

    pub async fn synthesize_summary(&self, text: &str) -> Result<Vec<u8>, PollyTtsError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(PollyTtsError::EmptyText);
        }

        // Polly's per-request text limits are much lower than typical summary length.
        // Chunk text conservatively and stitch audio reliably.
        //
        // Polly MP3 stitching via raw concatenation has proven fragile in browsers
        // (codecs/metadata boundaries can cause early decode termination), so we
        // stitch PCM and wrap it into a WAV container.
        //
        // `text` already contains injected SSML `<break .../>` tags, so we must
        // chunk without splitting those tags (otherwise Polly rejects the request
        // as `Invalid SSML request`).
        let chunks = split_ssml_for_polly(text, 2500);
        let mut pcm_audio = Vec::new();

        let sample_rate_u32 = self.sample_rate.parse::<u32>().unwrap_or(8000);
        let sample_rate_for_request = sample_rate_u32.to_string();

        for chunk in chunks {
            // Polly requires SSML input to be wrapped in a `<speak>` root element.
            // We send SSML per-chunk to keep text sizes bounded.
            let ssml = format!("<speak>{chunk}</speak>");
            let request = self
                .client
                .synthesize_speech()
                .text(ssml)
                .voice_id(self.voice_id.as_str().into())
                .engine(self.engine.as_str().into())
                .text_type(PollyTextType::Ssml)
                // Always request PCM for reliable concatenation.
                .output_format(PollyOutputFormat::Pcm)
                // Explicitly request the sample rate we will use for the WAV wrapper.
                .sample_rate(sample_rate_for_request.as_str());

            let response = request
                .send()
                .await
                .map_err(|err| PollyTtsError::Request(format!("{err:?}")))?;

            let stream = response.audio_stream;
            let bytes = stream
                .collect()
                .await
                .map_err(|err| PollyTtsError::Request(format!("{err:?}")))?
                .into_bytes();

            // PCM output is raw signed 16-bit little-endian mono.
            pcm_audio.extend_from_slice(&bytes);
        }

        Ok(wrap_pcm_s16le_mono_to_wav(pcm_audio, sample_rate_u32))
    }

    pub async fn resolve_voice_id_for_cache_key(&self) -> Result<String, PollyTtsError> {
        Ok(self.voice_id.clone())
    }

    pub fn model_id(&self) -> &str {
        &self.engine
    }

    pub fn output_format(&self) -> &str {
        &self.output_format
    }
}

fn split_ssml_for_polly(input: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    // Custom tokenizer that keeps tags <...> intact and splits everything else
    // into whitespace-separated words.
    let mut tokens = Vec::new();
    let mut in_tag = false;
    let mut token_start = 0;

    let chars: Vec<(usize, char)> = input.char_indices().collect();
    for (idx, c) in &chars {
        if *c == '<' {
            if !in_tag && *idx > token_start {
                // Add preceding text split into words.
                for word in input[token_start..*idx].split_whitespace() {
                    tokens.push(word);
                }
            }
            in_tag = true;
            token_start = *idx;
        } else if *c == '>' && in_tag {
            in_tag = false;
            tokens.push(&input[token_start..*idx + 1]);
            token_start = *idx + 1;
        } else if c.is_whitespace() && !in_tag {
            if *idx > token_start {
                tokens.push(&input[token_start..*idx]);
            }
            token_start = *idx + 1;
        }
    }

    if token_start < input.len() {
        if in_tag {
            tokens.push(&input[token_start..]);
        } else {
            for word in input[token_start..].split_whitespace() {
                tokens.push(word);
            }
        }
    }

    for token in tokens {
        let token_chars = token.chars().count();
        let next_len = if current.is_empty() {
            token_chars
        } else {
            current_len + 1 + token_chars
        };

        if !current.is_empty() && next_len > max_chars {
            chunks.push(current);
            current = String::new();
            current_len = 0;
        }

        if !current.is_empty() {
            current.push(' ');
            current_len += 1;
        }
        current.push_str(token);
        current_len += token_chars;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

/*
fn polly_output_format_for_request(wants_wav_or_pcm: bool) -> PollyOutputFormat {
    if wants_wav_or_pcm {
        PollyOutputFormat::Pcm
    } else {
        PollyOutputFormat::Mp3
    }
}
*/

fn wrap_pcm_s16le_mono_to_wav(pcm_s16le_mono: Vec<u8>, sample_rate: u32) -> Vec<u8> {
    // Polly returns raw PCM bytes for `output_format=pcm`:
    // - signed 16-bit little endian
    // - mono
    // We wrap it into a minimal WAV container so browsers can decode it reliably.
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;
    const BLOCK_ALIGN: u16 = (CHANNELS * BITS_PER_SAMPLE) / 8;
    let byte_rate: u32 = sample_rate * BLOCK_ALIGN as u32;

    let data_size: u32 = pcm_s16le_mono.len() as u32;
    let riff_chunk_size: u32 = 36 + data_size;

    let mut out = Vec::with_capacity(44 + pcm_s16le_mono.len());

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_chunk_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");

    // fmt chunk
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size
    out.extend_from_slice(&1u16.to_le_bytes()); // audio format PCM
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&BLOCK_ALIGN.to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());

    // data chunk
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    out.extend_from_slice(&pcm_s16le_mono);

    out
}

/*
fn strip_leading_id3v2(bytes: &[u8]) -> &[u8] {
    if bytes.len() < 10 || &bytes[0..3] != b"ID3" {
        return bytes;
    }

    // ID3v2 size is stored as synchsafe integer in bytes 6..10.
    let tag_size = ((bytes[6] as usize & 0x7f) << 21)
        | ((bytes[7] as usize & 0x7f) << 14)
        | ((bytes[8] as usize & 0x7f) << 7)
        | (bytes[9] as usize & 0x7f);
    let total_header = 10 + tag_size;

    if total_header >= bytes.len() {
        bytes
    } else {
        &bytes[total_header..]
    }
}

fn strip_trailing_id3v1(bytes: &[u8]) -> &[u8] {
    // ID3v1 tags are fixed-size 128-byte trailers that start with "TAG".
    // When Polly returns one per chunk, non-final chunk trailers can make
    // stitched streams appear truncated to some players.
    if bytes.len() < 128 {
        return bytes;
    }
    let trailer_start = bytes.len() - 128;
    if &bytes[trailer_start..trailer_start + 3] == b"TAG" {
        &bytes[..trailer_start]
    } else {
        bytes
    }
}
*/
