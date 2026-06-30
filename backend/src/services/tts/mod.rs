use google_cloud_texttospeech_v1::{
    client::TextToSpeech,
    model::{AudioConfig, AudioEncoding, SynthesisInput, VoiceSelectionParams},
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

fn split_ssml_for_tts(input: &str, max_chars: usize) -> Vec<String> {
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

fn wrap_pcm_s16le_mono_to_wav(pcm_s16le_mono: Vec<u8>, sample_rate: u32) -> Vec<u8> {
    // Cloud TTS PCM output is raw signed 16-bit little-endian mono.
    // Wrap it into a minimal WAV container so browsers can decode it reliably.
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

#[derive(Debug)]
pub struct TextToSpeechService {
    client: TextToSpeech,
    voice_name: String,
    language_code: String,
    model_name: Option<String>,
    request_audio_encoding: AudioEncoding,
    output_format: String,
    sample_rate_hertz: i32,
    wrap_pcm_as_wav: bool,
}

#[derive(Debug, Error)]
pub enum TextToSpeechError {
    #[error("summary text is empty")]
    EmptyText,
    #[error("failed to configure Google Cloud Text-to-Speech: {0}")]
    Config(String),
    #[error("failed to call Google Cloud Text-to-Speech API: {0}")]
    Request(String),
}

impl TextToSpeechService {
    pub async fn from_adc(
        voice_name: String,
        language_code: String,
        model_name: Option<String>,
        audio_encoding: String,
        sample_rate_hertz: i32,
    ) -> Result<Self, TextToSpeechError> {
        let client = TextToSpeech::builder()
            .build()
            .await
            .map_err(|err| TextToSpeechError::Config(err.to_string()))?;
        let (request_audio_encoding, output_format, wrap_pcm_as_wav) =
            parse_audio_encoding(&audio_encoding)?;
        Ok(Self {
            client,
            voice_name,
            language_code,
            model_name,
            request_audio_encoding,
            output_format,
            sample_rate_hertz,
            wrap_pcm_as_wav,
        })
    }

    pub async fn synthesize_summary(&self, text: &str) -> Result<Vec<u8>, TextToSpeechError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(TextToSpeechError::EmptyText);
        }

        let chunks = split_ssml_for_tts(text, 2500);
        let mut audio = Vec::new();

        for chunk in chunks {
            let ssml = format!("<speak>{chunk}</speak>");
            let mut voice = VoiceSelectionParams::new()
                .set_language_code(self.language_code.clone())
                .set_name(self.voice_name.clone());
            if let Some(model_name) = &self.model_name {
                voice = voice.set_model_name(model_name.clone());
            }
            let response = self
                .client
                .synthesize_speech()
                .set_input(SynthesisInput::new().set_ssml(ssml))
                .set_voice(voice)
                .set_audio_config(
                    AudioConfig::new()
                        .set_audio_encoding(self.request_audio_encoding.clone())
                        .set_sample_rate_hertz(self.sample_rate_hertz),
                )
                .send()
                .await
                .map_err(|err| TextToSpeechError::Request(format!("{err:?}")))?;

            audio.extend_from_slice(&response.audio_content);
        }

        if self.wrap_pcm_as_wav {
            Ok(wrap_pcm_s16le_mono_to_wav(
                audio,
                self.sample_rate_hertz as u32,
            ))
        } else {
            Ok(audio)
        }
    }

    pub async fn resolve_voice_id_for_cache_key(&self) -> Result<String, TextToSpeechError> {
        Ok(self.voice_name.clone())
    }

    pub fn model_id(&self) -> &str {
        self.model_name.as_deref().unwrap_or("google-cloud-tts")
    }

    pub fn output_format(&self) -> &str {
        &self.output_format
    }
}

fn parse_audio_encoding(value: &str) -> Result<(AudioEncoding, String, bool), TextToSpeechError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "LINEAR16" | "WAV" | "PCM" => Ok((AudioEncoding::Pcm, "wav".to_string(), true)),
        "MP3" => Ok((AudioEncoding::Mp3, "mp3".to_string(), false)),
        other => Err(TextToSpeechError::Config(format!(
            "unsupported GOOGLE_TTS_AUDIO_ENCODING `{other}`; use LINEAR16, PCM, WAV, or MP3"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_audio_encoding_uses_pcm_for_wav_output() {
        let (encoding, output_format, wrap) = parse_audio_encoding("LINEAR16").unwrap();
        assert_eq!(encoding, AudioEncoding::Pcm);
        assert_eq!(output_format, "wav");
        assert!(wrap);
    }

    #[test]
    fn split_ssml_preserves_tags() {
        let chunks = split_ssml_for_tts(r#"Hello <break time="0.6s" /> world"#, 12);
        assert!(
            chunks
                .iter()
                .any(|chunk| chunk == r#"<break time="0.6s" />"#)
        );
    }

    #[test]
    fn wav_wrapper_adds_riff_header() {
        let wav = wrap_pcm_s16le_mono_to_wav(vec![0, 0, 1, 0], 16_000);
        assert!(wav.starts_with(b"RIFF"));
        assert_eq!(&wav[8..12], b"WAVE");
    }
}
