use crate::modules::auth::application as auth_application;
use crate::modules::copilot::domain::{
    CopilotAnswer, CopilotContext, CopilotMode, ScreenshotAttachment,
};
use crate::modules::copilot::infrastructure;
use crate::modules::settings::domain::AppSettings;
use crate::support::openai::codex_responses::{
    CodexAuth, CodexInputItem, CodexResponsesClient, CodexTextRequest,
};

const DEFAULT_COPILOT_MAX_TRANSCRIPT_CHARS: usize = 6_000;

pub fn answer_question(
    settings: &AppSettings,
    context: CopilotContext,
    thread_id: Option<i64>,
) -> Result<CopilotAnswer, String> {
    let question = context.question.trim();
    if question.is_empty() {
        return Err(String::from(
            "Escreva uma pergunta antes de chamar o copiloto.",
        ));
    }

    let session = auth_application::load_or_refresh_session()?;
    let account_id = session
        .account_id
        .as_deref()
        .ok_or_else(|| String::from("Sem account_id no token OAuth. Faca login novamente."))?;

    let instructions = build_instructions(context.mode);
    let input_items = build_input_items(
        &context,
        settings.copilot_auto_include_transcript,
        DEFAULT_COPILOT_MAX_TRANSCRIPT_CHARS,
    );

    let client = CodexResponsesClient::new()?;
    let answer = client.generate_text(
        CodexAuth {
            bearer_token: session.bearer_token(),
            account_id,
        },
        CodexTextRequest {
            model: &settings.copilot_model,
            instructions: &instructions,
            input: input_items,
        },
    )?;

    let persisted_thread_id = if settings.copilot_save_history {
        let screenshot_mime = context
            .screenshot
            .as_ref()
            .map(|item| item.mime_type.as_str());
        let screenshot_bytes = context
            .screenshot
            .as_ref()
            .map(|item| item.bytes.len())
            .unwrap_or(0);
        let ensured_thread_id =
            infrastructure::ensure_thread(thread_id, context.session_id, context.mode)?;
        infrastructure::append_turn(
            ensured_thread_id,
            context.mode,
            question,
            &answer,
            screenshot_mime,
            screenshot_bytes,
        )?;
        Some(ensured_thread_id)
    } else {
        thread_id
    };

    Ok(CopilotAnswer {
        answer,
        thread_id: persisted_thread_id,
    })
}

fn build_instructions(mode: CopilotMode) -> String {
    let base = "You are OpenVoice Copilot. Use only the provided context plus the user question. Be concrete, concise, and do not invent facts that are not grounded in transcript or image context.";

    let mode_instructions = match mode {
        CopilotMode::General => {
            "Answer directly. If the context is incomplete, say what is missing."
        }
        CopilotMode::Interview => {
            "Optimize for technical interview help. Prefer: problem framing, edge cases, brute-force to optimal path, tradeoffs, concise hints, and short 'what to say next' guidance."
        }
        CopilotMode::Meeting => {
            "Optimize for meetings and agendas. Prefer: decisions, action items, blockers, unanswered questions, follow-ups, and note-friendly structure."
        }
    };

    format!("{base} {mode_instructions}")
}

fn build_input_items(
    context: &CopilotContext,
    include_transcript: bool,
    max_transcript_chars: usize,
) -> Vec<CodexInputItem> {
    let mut blocks = Vec::new();

    let mut prompt = String::new();
    prompt.push_str(&format!("Mode: {}\n", context.mode.label()));

    if let Some(label) = context.session_label.as_deref() {
        prompt.push_str(&format!("Context source: {label}\n"));
    }

    if include_transcript {
        let transcript =
            build_truncated_transcript(&context.transcript_segments, max_transcript_chars);
        if !transcript.is_empty() {
            prompt.push_str("\nTranscript context:\n");
            prompt.push_str(&transcript);
            prompt.push_str("\n");
        }
    }

    prompt.push_str("\nUser question:\n");
    prompt.push_str(context.question.trim());

    blocks.push(CodexInputItem::text(prompt));

    if let Some(screenshot) = context.screenshot.as_ref() {
        blocks.push(CodexInputItem::image_data_url(
            &screenshot.mime_type,
            &screenshot.bytes,
        ));
    }

    blocks
}

fn build_truncated_transcript(segments: &[String], max_chars: usize) -> String {
    let mut result = String::new();
    let mut used_chars = 0;

    for segment in segments.iter().rev() {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }

        let chars = trimmed.chars().count();
        let separator = if result.is_empty() { 0 } else { 1 };
        if used_chars + chars + separator > max_chars {
            break;
        }

        if result.is_empty() {
            result.insert_str(0, trimmed);
        } else {
            result.insert(0, '\n');
            result.insert_str(0, trimmed);
        }

        used_chars += chars + separator;
    }

    result
}

pub fn screenshot_summary(screenshot: &ScreenshotAttachment) -> String {
    format!(
        "{} · {:.1} KB",
        screenshot.mime_type,
        screenshot.bytes.len() as f32 / 1024.0
    )
}

#[cfg(test)]
mod tests {
    use super::{build_input_items, build_truncated_transcript};
    use crate::modules::copilot::domain::{CopilotContext, CopilotMode};

    #[test]
    fn transcript_keeps_recent_segments_within_limit() {
        let transcript = build_truncated_transcript(
            &[
                String::from("first"),
                String::from("second"),
                String::from("third"),
            ],
            12,
        );

        assert_eq!(transcript, "second\nthird");
    }

    #[test]
    fn input_items_include_transcript_and_mode() {
        let context = CopilotContext {
            mode: CopilotMode::Meeting,
            question: String::from("What changed?"),
            transcript_segments: vec![String::from("Decision made")],
            session_id: Some(7),
            session_label: Some(String::from("Sessao #7")),
            screenshot: None,
        };

        let items = build_input_items(&context, true, 400);
        let text = items[0].as_text().expect("text block");

        assert!(text.contains("Mode: Meeting"));
        assert!(text.contains("Decision made"));
        assert!(text.contains("What changed?"));
    }
}
