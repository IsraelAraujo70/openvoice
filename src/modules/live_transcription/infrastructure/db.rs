use rusqlite::{params, Connection};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: i64,
    pub started_at: String,
    pub stopped_at: Option<String>,
    pub language: Option<String>,
    pub model: Option<String>,
    pub segment_count: i64,
    pub preview: String,
}

// ---------------------------------------------------------------------------
// DB path
// ---------------------------------------------------------------------------

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("openvoice")
        .join("openvoice.db")
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

pub fn open_db() -> Result<Connection, String> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Nao consegui criar o diretorio do banco: {e}"))?;
    }
    Connection::open(&path).map_err(|e| format!("Nao consegui abrir o banco SQLite: {e}"))
}

pub fn ensure_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS lt_sessions (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at    TEXT NOT NULL,
            stopped_at    TEXT,
            language      TEXT,
            model         TEXT,
            segment_count INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS lt_segments (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id   INTEGER NOT NULL REFERENCES lt_sessions(id),
            position     INTEGER NOT NULL,
            item_id      TEXT NOT NULL DEFAULT '',
            transcript   TEXT NOT NULL,
            completed_at TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("Nao consegui criar o schema SQLite: {e}"))
}

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

pub fn save_session(
    segments: Vec<String>,
    started_at: String,
    stopped_at: String,
    language: Option<String>,
    model: Option<String>,
) -> Result<i64, String> {
    let conn = open_db()?;
    ensure_schema(&conn)?;

    conn.execute(
        "INSERT INTO lt_sessions (started_at, stopped_at, language, model, segment_count)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            started_at,
            stopped_at,
            language,
            model,
            segments.len() as i64
        ],
    )
    .map_err(|e| format!("Nao consegui inserir a sessao: {e}"))?;

    let session_id = conn.last_insert_rowid();
    let now = now_iso();

    for (i, transcript) in segments.iter().enumerate() {
        conn.execute(
            "INSERT INTO lt_segments (session_id, position, item_id, transcript, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, i as i64, "", transcript, now],
        )
        .map_err(|e| format!("Nao consegui inserir o segmento {i}: {e}"))?;
    }

    Ok(session_id)
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

pub fn list_sessions() -> Result<Vec<SessionSummary>, String> {
    let conn = open_db()?;
    ensure_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.started_at, s.stopped_at, s.language, s.model, s.segment_count,
                    COALESCE(
                        (SELECT substr(seg.transcript, 1, 90)
                         FROM lt_segments seg
                         WHERE seg.session_id = s.id
                         ORDER BY seg.position ASC
                         LIMIT 1),
                        ''
                    ) AS preview
             FROM lt_sessions s
             ORDER BY s.id DESC",
        )
        .map_err(|e| format!("Nao consegui preparar a query de sessoes: {e}"))?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                started_at: row.get(1)?,
                stopped_at: row.get(2)?,
                language: row.get(3)?,
                model: row.get(4)?,
                segment_count: row.get(5)?,
                preview: row.get(6)?,
            })
        })
        .map_err(|e| format!("Nao consegui executar a query de sessoes: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sessions)
}

pub fn get_session_segments(session_id: i64) -> Result<Vec<String>, String> {
    let conn = open_db()?;
    ensure_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT transcript FROM lt_segments
             WHERE session_id = ?1
             ORDER BY position ASC",
        )
        .map_err(|e| format!("Nao consegui preparar a query de segmentos: {e}"))?;

    let segments = stmt
        .query_map(params![session_id], |row| row.get(0))
        .map_err(|e| format!("Nao consegui executar a query de segmentos: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(segments)
}

// ---------------------------------------------------------------------------
// Timestamp helpers
// ---------------------------------------------------------------------------

pub fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    unix_secs_to_iso(secs)
}

fn unix_secs_to_iso(secs: u64) -> String {
    let s = secs % 60;
    let mins = secs / 60;
    let mi = mins % 60;
    let hours = mins / 60;
    let h = hours % 24;
    let total_days = hours / 24;

    let (year, month, day) = days_to_ymd(total_days);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let in_year = if is_leap(year) { 366 } else { 365 };
        if days < in_year {
            break;
        }
        days -= in_year;
        year += 1;
    }

    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0u64;
    for &dim in &month_days {
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }

    (year, month + 1, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Format an ISO timestamp into a human-readable string for display.
/// Input: "2026-03-11T14:32:00Z"
/// Output: "11 Mar 2026, 14:32"
pub fn format_iso_for_display(iso: &str) -> String {
    let months = [
        "Jan", "Fev", "Mar", "Abr", "Mai", "Jun", "Jul", "Ago", "Set", "Out", "Nov", "Dez",
    ];

    let date_part = iso.split('T').next().unwrap_or(iso);
    let time_part = iso.split('T').nth(1).unwrap_or("").trim_end_matches('Z');

    let mut parts = date_part.split('-');
    let year = parts.next().unwrap_or("?");
    let month_num: usize = parts.next().unwrap_or("1").parse().unwrap_or(1);
    let day = parts.next().unwrap_or("?");

    let month_name = months.get(month_num.saturating_sub(1)).unwrap_or(&"?");
    let hhmm = &time_part[..time_part.len().min(5)];

    format!("{day} {month_name} {year}, {hhmm}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_roundtrip_epoch() {
        assert_eq!(unix_secs_to_iso(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn iso_known_date() {
        // 2026-03-11T14:32:00Z
        let secs: u64 = 1741700720;
        let result = unix_secs_to_iso(secs);
        assert!(
            result.starts_with("2026-"),
            "expected 2026 year, got {result}"
        );
    }

    #[test]
    fn display_format() {
        let display = format_iso_for_display("2026-03-11T14:32:00Z");
        assert_eq!(display, "11 Mar 2026, 14:32");
    }
}
