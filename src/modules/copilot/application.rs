use crate::modules::auth::application as auth_application;
use crate::modules::copilot::domain::{
    CopilotAnswer, CopilotChatMessage, CopilotContext, CopilotHistoryMessage, CopilotMode,
    CopilotRole, CopilotThreadSummary, ScreenshotAttachment,
};
use crate::modules::copilot::infrastructure;
use crate::modules::settings::domain::AppSettings;
use crate::support::openai::codex_responses::{
    CodexAuth, CodexInputItem, CodexResponsesClient, CodexTextRequest,
};
use std::sync::{Arc, Mutex, mpsc};

const DEFAULT_COPILOT_MAX_TRANSCRIPT_CHARS: usize = 6_000;
const DEFAULT_COPILOT_MAX_HISTORY_CHARS: usize = 4_000;

pub type SharedReceiver = Arc<Mutex<mpsc::Receiver<RuntimeEvent>>>;

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    Delta(String),
    Completed {
        answer: String,
        thread_id: Option<i64>,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ActiveCopilotStream {
    receiver: SharedReceiver,
}

impl ActiveCopilotStream {
    pub fn receiver(&self) -> SharedReceiver {
        Arc::clone(&self.receiver)
    }
}

pub fn start_answer_stream(
    settings: AppSettings,
    context: CopilotContext,
    thread_id: Option<i64>,
) -> Result<ActiveCopilotStream, String> {
    let question = resolve_question(&context);
    if question.trim().is_empty() {
        return Err(String::from(
            "Escreva uma pergunta ou anexe um screenshot antes de chamar o copiloto.",
        ));
    }

    let (sender, receiver) = mpsc::channel();
    let shared = Arc::new(Mutex::new(receiver));

    std::thread::spawn(move || {
        let result = answer_question_inner(&settings, context, thread_id, |delta| {
            let _ = sender.send(RuntimeEvent::Delta(delta.to_owned()));
        });

        match result {
            Ok(answer) => {
                let _ = sender.send(RuntimeEvent::Completed {
                    answer: answer.answer,
                    thread_id: answer.thread_id,
                });
            }
            Err(error) => {
                let _ = sender.send(RuntimeEvent::Error(error));
            }
        }
    });

    Ok(ActiveCopilotStream { receiver: shared })
}

#[derive(Debug, Clone)]
pub struct LoadedCopilotThread {
    pub summary: CopilotThreadSummary,
    pub messages: Vec<CopilotChatMessage>,
}

pub fn poll_next_event(receiver: SharedReceiver) -> Option<RuntimeEvent> {
    receiver.lock().ok()?.recv().ok()
}

#[allow(dead_code)]
pub fn answer_question(
    settings: &AppSettings,
    context: CopilotContext,
    thread_id: Option<i64>,
) -> Result<CopilotAnswer, String> {
    answer_question_inner(settings, context, thread_id, |_| {})
}

fn answer_question_inner(
    settings: &AppSettings,
    context: CopilotContext,
    thread_id: Option<i64>,
    mut on_delta: impl FnMut(&str),
) -> Result<CopilotAnswer, String> {
    let question = resolve_question(&context);

    let session = auth_application::load_or_refresh_session()?;
    let account_id = session
        .account_id
        .as_deref()
        .ok_or_else(|| String::from("Sem account_id no token OAuth. Faca login novamente."))?;

    let instructions = build_instructions(context.mode);
    let input_items = build_input_items(
        &context,
        &question,
        settings.copilot_auto_include_transcript,
        DEFAULT_COPILOT_MAX_TRANSCRIPT_CHARS,
        DEFAULT_COPILOT_MAX_HISTORY_CHARS,
    );

    let client = CodexResponsesClient::new()?;
    let answer = client.generate_text_streaming(
        CodexAuth {
            bearer_token: session.bearer_token(),
            account_id,
        },
        CodexTextRequest {
            model: &settings.copilot_model,
            instructions: &instructions,
            input: input_items,
        },
        |delta| on_delta(delta),
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
            &question,
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
    let base = "You are OpenVoice Copilot. Use only the provided context plus the user question. If an image is attached, inspect it carefully and treat visible text/code/problem statements as valid context. Do not ignore the screenshot. Do not invent hidden details, but do make a best effort from what is visible.";

    let mode_instructions = match mode {
        CopilotMode::General => {
            "Answer directly. If the context is incomplete, say what is missing. If the screenshot contains a visible UI, document, code, or problem statement, reason from it explicitly."
        }
        CopilotMode::Interview => {
            "Optimize for technical interview help. If the screenshot or prompt contains a coding problem, provide a real solution instead of generic advice. Prefer this structure: 1) short approach, 2) edge cases, 3) complexity, 4) full runnable solution in markdown code block, 5) short explanation. If some details are partially obscured, state the assumption briefly and still provide the best concrete solution you can."
        }
        CopilotMode::Meeting => {
            "Optimize for meetings and agendas. Prefer: decisions, action items, blockers, unanswered questions, follow-ups, and note-friendly structure."
        }
    };

    format!("{base} {mode_instructions}")
}

fn build_input_items(
    context: &CopilotContext,
    question: &str,
    include_transcript: bool,
    max_transcript_chars: usize,
    max_history_chars: usize,
) -> Vec<CodexInputItem> {
    let mut blocks = Vec::new();

    let mut prompt = String::new();
    prompt.push_str(&format!("Mode: {}\n", context.mode.label()));

    if let Some(label) = context.session_label.as_deref() {
        prompt.push_str(&format!("Context source: {label}\n"));
    }

    if context.screenshot.is_some() {
        prompt.push_str("A screenshot is attached. Read the screenshot carefully and use it as primary evidence when relevant.\n");
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

    let history = build_truncated_history(&context.history_messages, max_history_chars);
    if !history.is_empty() {
        prompt.push_str("\nConversation history:\n");
        prompt.push_str(&history);
        prompt.push_str("\n");
    }

    prompt.push_str("\nUser question:\n");
    prompt.push_str(question.trim());

    blocks.push(CodexInputItem::text(prompt));

    if let Some(screenshot) = context.screenshot.as_ref() {
        blocks.push(CodexInputItem::image_data_url(
            &screenshot.mime_type,
            &screenshot.bytes,
        ));
    }

    blocks
}

fn resolve_question(context: &CopilotContext) -> String {
    let question = context.question.trim();

    if !question.is_empty() {
        return question.to_owned();
    }

    if context.screenshot.is_some() {
        return match context.mode {
            CopilotMode::Interview => String::from(
                "Analyze the screenshot and solve the coding/interview task shown on screen. Give the best concrete solution you can from the visible context.",
            ),
            CopilotMode::Meeting => String::from(
                "Analyze the screenshot and tell me the most important meeting-ready response or next step based on what is visible.",
            ),
            CopilotMode::General => {
                String::from("Analyze the screenshot and help me with what is on screen.")
            }
        };
    }

    String::new()
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

fn build_truncated_history(messages: &[CopilotHistoryMessage], max_chars: usize) -> String {
    let mut lines = Vec::new();
    let mut used_chars = 0;

    for message in messages.iter().rev() {
        let trimmed = message.content.trim();
        if trimmed.is_empty() {
            continue;
        }

        let role = match message.role {
            CopilotRole::User => "User",
            CopilotRole::Assistant => "Assistant",
        };
        let line = format!("{role}: {trimmed}");
        let chars = line.chars().count();
        let separator = if lines.is_empty() { 0 } else { 1 };

        if used_chars + chars + separator > max_chars {
            break;
        }

        lines.push(line);
        used_chars += chars + separator;
    }

    lines.reverse();
    lines.join("\n")
}

pub fn screenshot_summary(screenshot: &ScreenshotAttachment) -> String {
    format!(
        "{} · {:.1} KB",
        screenshot.mime_type,
        screenshot.bytes.len() as f32 / 1024.0
    )
}

pub fn list_saved_threads() -> Result<Vec<CopilotThreadSummary>, String> {
    infrastructure::list_threads()
}

pub fn load_saved_thread(thread_id: i64) -> Result<LoadedCopilotThread, String> {
    let summary = infrastructure::list_threads()?
        .into_iter()
        .find(|thread| thread.id == thread_id)
        .ok_or_else(|| format!("Thread do copilot #{thread_id} nao encontrada."))?;

    let turns = infrastructure::load_turns(thread_id)?;
    let mut messages = Vec::with_capacity(turns.len() * 2);

    for turn in turns {
        messages.push(CopilotChatMessage::user(turn.question));

        let mut assistant = CopilotChatMessage::assistant_streaming();
        assistant.replace_content(turn.answer, false);
        messages.push(assistant);
    }

    Ok(LoadedCopilotThread { summary, messages })
}

#[cfg(test)]
mod tests {
    use super::{
        build_input_items, build_truncated_history, build_truncated_transcript, resolve_question,
    };
    use crate::modules::copilot::domain::{
        CopilotContext, CopilotHistoryMessage, CopilotMode, CopilotRole,
    };

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
            history_messages: Vec::new(),
            transcript_segments: vec![String::from("Decision made")],
            session_id: Some(7),
            session_label: Some(String::from("Sessao #7")),
            screenshot: None,
        };

        let items = build_input_items(&context, context.question.as_str(), true, 400, 400);
        let text = items[0].as_text().expect("text block");

        assert!(text.contains("Mode: Meeting"));
        assert!(text.contains("Decision made"));
        assert!(text.contains("What changed?"));
    }

    #[test]
    fn screenshot_without_question_gets_interview_fallback() {
        let context = CopilotContext {
            mode: CopilotMode::Interview,
            question: String::new(),
            history_messages: Vec::new(),
            transcript_segments: Vec::new(),
            session_id: None,
            session_label: None,
            screenshot: Some(crate::modules::copilot::domain::ScreenshotAttachment {
                bytes: vec![1, 2, 3],
                mime_type: String::from("image/png"),
            }),
        };

        let resolved = resolve_question(&context);

        assert!(resolved.contains("solve the coding/interview task"));
    }

    #[test]
    fn history_keeps_recent_messages_within_limit() {
        let history = build_truncated_history(
            &[
                CopilotHistoryMessage {
                    role: CopilotRole::User,
                    content: String::from("primeira"),
                },
                CopilotHistoryMessage {
                    role: CopilotRole::Assistant,
                    content: String::from("segunda"),
                },
                CopilotHistoryMessage {
                    role: CopilotRole::User,
                    content: String::from("terceira"),
                },
            ],
            80,
        );

        assert!(history.contains("Assistant: segunda"));
        assert!(history.contains("User: terceira"));
    }
}
