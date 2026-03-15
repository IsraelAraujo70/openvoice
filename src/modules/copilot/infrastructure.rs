use crate::modules::copilot::domain::{
    CopilotMode, CopilotThread, CopilotThreadSummary, CopilotTurn,
};
use crate::modules::live_transcription::infrastructure::db;
use rusqlite::{params, Connection};

pub fn ensure_schema(conn: &Connection) -> Result<(), String> {
    db::ensure_schema(conn)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS cp_threads (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER REFERENCES lt_sessions(id),
            mode       TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cp_turns (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            thread_id       INTEGER NOT NULL REFERENCES cp_threads(id),
            mode            TEXT NOT NULL,
            question        TEXT NOT NULL,
            answer          TEXT NOT NULL,
            screenshot_mime TEXT,
            screenshot_bytes INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_cp_threads_session_id
        ON cp_threads(session_id);
        CREATE INDEX IF NOT EXISTS idx_cp_turns_thread_id
        ON cp_turns(thread_id);",
    )
    .map_err(|error| format!("Nao consegui criar schema do copilot: {error}"))
}

pub fn create_thread(session_id: Option<i64>, mode: CopilotMode) -> Result<CopilotThread, String> {
    let conn = db::open_db()?;
    ensure_schema(&conn)?;
    create_thread_in_conn(&conn, session_id, mode)
}

pub fn append_turn(
    thread_id: i64,
    mode: CopilotMode,
    question: &str,
    answer: &str,
    screenshot_mime: Option<&str>,
    screenshot_bytes: usize,
) -> Result<CopilotTurn, String> {
    let conn = db::open_db()?;
    ensure_schema(&conn)?;
    append_turn_in_conn(
        &conn,
        thread_id,
        mode,
        question,
        answer,
        screenshot_mime,
        screenshot_bytes,
    )
}

fn create_thread_in_conn(
    conn: &Connection,
    session_id: Option<i64>,
    mode: CopilotMode,
) -> Result<CopilotThread, String> {
    let created_at = db::now_iso();

    conn.execute(
        "INSERT INTO cp_threads (session_id, mode, created_at)
         VALUES (?1, ?2, ?3)",
        params![session_id, mode.code(), created_at],
    )
    .map_err(|error| format!("Nao consegui criar thread do copilot: {error}"))?;

    Ok(CopilotThread {
        id: conn.last_insert_rowid(),
        session_id,
        mode,
        created_at,
    })
}

fn append_turn_in_conn(
    conn: &Connection,
    thread_id: i64,
    mode: CopilotMode,
    question: &str,
    answer: &str,
    screenshot_mime: Option<&str>,
    screenshot_bytes: usize,
) -> Result<CopilotTurn, String> {
    let created_at = db::now_iso();

    conn.execute(
        "INSERT INTO cp_turns (
            thread_id, mode, question, answer, screenshot_mime, screenshot_bytes, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            thread_id,
            mode.code(),
            question.trim(),
            answer.trim(),
            screenshot_mime,
            screenshot_bytes as i64,
            created_at
        ],
    )
    .map_err(|error| format!("Nao consegui salvar turno do copilot: {error}"))?;

    Ok(CopilotTurn {
        id: conn.last_insert_rowid(),
        thread_id,
        mode,
        question: question.trim().to_owned(),
        answer: answer.trim().to_owned(),
        screenshot_mime: screenshot_mime.map(str::to_owned),
        screenshot_bytes,
        created_at,
    })
}

pub fn ensure_thread(
    thread_id: Option<i64>,
    session_id: Option<i64>,
    mode: CopilotMode,
) -> Result<i64, String> {
    match thread_id {
        Some(id) => Ok(id),
        None => create_thread(session_id, mode).map(|thread| thread.id),
    }
}

pub fn list_threads() -> Result<Vec<CopilotThreadSummary>, String> {
    let conn = db::open_db()?;
    ensure_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT
                t.id,
                t.session_id,
                t.mode,
                t.created_at,
                COUNT(turns.id) AS turn_count,
                COALESCE(
                    (
                        SELECT question
                        FROM cp_turns
                        WHERE thread_id = t.id
                        ORDER BY id DESC
                        LIMIT 1
                    ),
                    ''
                ) AS last_preview
             FROM cp_threads t
             LEFT JOIN cp_turns turns ON turns.thread_id = t.id
             GROUP BY t.id, t.session_id, t.mode, t.created_at
             ORDER BY COALESCE(MAX(turns.created_at), t.created_at) DESC, t.id DESC",
        )
        .map_err(|error| {
            format!("Nao consegui preparar consulta de threads do copilot: {error}")
        })?;

    let rows = stmt
        .query_map([], |row| {
            let mode_code: String = row.get(2)?;

            Ok(CopilotThreadSummary {
                id: row.get(0)?,
                session_id: row.get(1)?,
                mode: CopilotMode::from_code(&mode_code),
                created_at: row.get(3)?,
                turn_count: row.get::<_, i64>(4)?.max(0) as usize,
                last_preview: row.get(5)?,
            })
        })
        .map_err(|error| format!("Nao consegui listar threads do copilot: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Nao consegui ler threads do copilot: {error}"))
}

pub fn load_turns(thread_id: i64) -> Result<Vec<CopilotTurn>, String> {
    let conn = db::open_db()?;
    ensure_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, thread_id, mode, question, answer, screenshot_mime, screenshot_bytes, created_at
             FROM cp_turns
             WHERE thread_id = ?1
             ORDER BY id ASC",
        )
        .map_err(|error| format!("Nao consegui preparar leitura de turnos do copilot: {error}"))?;

    let rows = stmt
        .query_map(params![thread_id], |row| {
            let mode_code: String = row.get(2)?;
            Ok(CopilotTurn {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                mode: CopilotMode::from_code(&mode_code),
                question: row.get(3)?,
                answer: row.get(4)?,
                screenshot_mime: row.get(5)?,
                screenshot_bytes: row.get::<_, i64>(6)?.max(0) as usize,
                created_at: row.get(7)?,
            })
        })
        .map_err(|error| format!("Nao consegui consultar turnos do copilot: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Nao consegui ler turnos do copilot: {error}"))
}

pub fn delete_thread(thread_id: i64) -> Result<(), String> {
    let conn = db::open_db()?;
    ensure_schema(&conn)?;

    conn.execute(
        "DELETE FROM cp_turns WHERE thread_id = ?1",
        params![thread_id],
    )
    .map_err(|error| format!("Nao consegui remover turnos do copilot: {error}"))?;

    conn.execute("DELETE FROM cp_threads WHERE id = ?1", params![thread_id])
        .map_err(|error| format!("Nao consegui remover thread do copilot: {error}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_schema;
    use crate::modules::copilot::domain::CopilotMode;
    use rusqlite::Connection;

    #[test]
    fn creates_schema_and_persists_turns() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        ensure_schema(&conn).expect("schema");

        conn.execute(
            "INSERT INTO lt_sessions (started_at, segment_count) VALUES ('2026-01-01T00:00:00Z', 0)",
            [],
        )
        .expect("seed session");

        let thread =
            super::create_thread_in_conn(&conn, Some(1), CopilotMode::Interview).expect("thread");
        let turn = super::append_turn_in_conn(
            &conn,
            thread.id,
            CopilotMode::Interview,
            "Q",
            "A",
            Some("image/png"),
            128,
        )
        .expect("turn");

        assert_eq!(turn.thread_id, thread.id);
        assert_eq!(turn.screenshot_bytes, 128);
    }
}
