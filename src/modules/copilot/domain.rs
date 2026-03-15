#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopilotMode {
    General,
    Interview,
    Meeting,
}

impl CopilotMode {
    pub fn code(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Interview => "interview",
            Self::Meeting => "meeting",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Interview => "Interview",
            Self::Meeting => "Meeting",
        }
    }

    pub fn from_code(value: &str) -> Self {
        match value.trim() {
            "interview" => Self::Interview,
            "meeting" => Self::Meeting,
            _ => Self::General,
        }
    }
}

impl std::fmt::Display for CopilotMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopilotRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct CopilotChatMessage {
    pub role: CopilotRole,
    pub content: String,
    pub markdown_items: Vec<markdown::Item>,
    pub is_streaming: bool,
}

impl CopilotChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: CopilotRole::User,
            content: content.into(),
            markdown_items: Vec::new(),
            is_streaming: false,
        }
    }

    pub fn assistant_streaming() -> Self {
        Self {
            role: CopilotRole::Assistant,
            content: String::new(),
            markdown_items: Vec::new(),
            is_streaming: true,
        }
    }

    pub fn replace_content(&mut self, content: impl Into<String>, is_streaming: bool) {
        self.content = content.into();
        self.markdown_items = markdown::parse(&self.content).collect();
        self.is_streaming = is_streaming;
    }

    pub fn append_delta(&mut self, delta: &str) {
        self.content.push_str(delta);
        self.markdown_items = markdown::parse(&self.content).collect();
        self.is_streaming = true;
    }
}

#[derive(Debug, Clone)]
pub struct ScreenshotAttachment {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct CopilotContext {
    pub mode: CopilotMode,
    pub question: String,
    pub history_messages: Vec<CopilotHistoryMessage>,
    pub transcript_segments: Vec<String>,
    pub session_id: Option<i64>,
    pub session_label: Option<String>,
    pub screenshot: Option<ScreenshotAttachment>,
}

#[derive(Debug, Clone)]
pub struct CopilotHistoryMessage {
    pub role: CopilotRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CopilotAnswer {
    pub answer: String,
    pub thread_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CopilotThread {
    pub id: i64,
    pub session_id: Option<i64>,
    pub mode: CopilotMode,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CopilotThreadSummary {
    pub id: i64,
    pub session_id: Option<i64>,
    pub mode: CopilotMode,
    pub created_at: String,
    pub turn_count: usize,
    pub last_preview: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CopilotTurn {
    pub id: i64,
    pub thread_id: i64,
    pub mode: CopilotMode,
    pub question: String,
    pub answer: String,
    pub screenshot_mime: Option<String>,
    pub screenshot_bytes: usize,
    pub created_at: String,
}
use iced::widget::markdown;
