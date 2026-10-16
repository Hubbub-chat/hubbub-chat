use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::errors::StoreError;
use hubbub_domain::entities::conversation::{Message, MessagePart, MessageRole};

pub async fn append(pool: &SqlitePool, msg: &Message) -> Result<(), StoreError> {
    let id_str = msg.id.to_string();
    let conv_id_str = msg.conversation_id.to_string();
    let run_id_str = msg.run_id.map(|r| r.to_string());
    let role_str = match msg.role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
    };
    let parts_json = serde_json::to_string(&msg.parts)?;
    let created_at_str = msg.created_at.to_rfc3339();

    // 1. Insert into messages table
    sqlx::query(
        r#"
        INSERT INTO messages (id, conversation_id, run_id, role, parts_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id_str)
    .bind(&conv_id_str)
    .bind(run_id_str)
    .bind(role_str)
    .bind(&parts_json)
    .bind(&created_at_str)
    .execute(pool)
    .await?;

    // 2. Extract text and insert into FTS5 index
    let text_content = extract_text_content(&msg.parts);
    if !text_content.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO messages_fts (message_id, conversation_id, content)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&id_str)
        .bind(&conv_id_str)
        .bind(&text_content)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn list(
    pool: &SqlitePool,
    conversation_id: Uuid,
    limit: u32,
    offset: u32,
) -> Result<Vec<Message>, StoreError> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let conv_id_str = conversation_id.to_string();
    let rows = sqlx::query(
        r#"
        SELECT id, conversation_id, run_id, role, parts_json, created_at
        FROM messages
        WHERE conversation_id = ?
        ORDER BY created_at ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(conv_id_str)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(parse_message_row(&row)?);
    }

    Ok(result)
}

/// Newest `limit` messages, oldest-first. `id` (UUIDv7) breaks `created_at` ties.
pub async fn list_recent(
    pool: &SqlitePool,
    conversation_id: Uuid,
    limit: u32,
) -> Result<Vec<Message>, StoreError> {
    let rows = sqlx::query(
        r#"
        SELECT id, conversation_id, run_id, role, parts_json, created_at
        FROM (
            SELECT id, conversation_id, run_id, role, parts_json, created_at
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at DESC, id DESC
            LIMIT ?
        )
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(conversation_id.to_string())
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    rows.iter().map(parse_message_row).collect()
}

pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<Message>, StoreError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let fts_query = format_fts_query(trimmed);

    let rows = sqlx::query(
        r#"
        SELECT m.id, m.conversation_id, m.run_id, m.role, m.parts_json, m.created_at
        FROM messages m
        JOIN messages_fts fts ON m.id = fts.message_id
        WHERE messages_fts MATCH ?
        ORDER BY fts.rank
        LIMIT 50
        "#,
    )
    .bind(fts_query)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(parse_message_row(&row)?);
    }

    Ok(result)
}

fn parse_message_row(row: &sqlx::sqlite::SqliteRow) -> Result<Message, StoreError> {
    let id_str: String = row.try_get("id")?;
    let conv_id_str: String = row.try_get("conversation_id")?;
    let run_id_str: Option<String> = row.try_get("run_id")?;
    let role_str: String = row.try_get("role")?;
    let parts_json: String = row.try_get("parts_json")?;
    let created_at_str: String = row.try_get("created_at")?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| StoreError::InvalidData(format!("Invalid UUID: {e}")))?;
    let conversation_id = Uuid::parse_str(&conv_id_str)
        .map_err(|e| StoreError::InvalidData(format!("Invalid conversation_id UUID: {e}")))?;
    let run_id = match run_id_str {
        Some(s) => Some(
            Uuid::parse_str(&s)
                .map_err(|e| StoreError::InvalidData(format!("Invalid run_id UUID: {e}")))?,
        ),
        None => None,
    };

    let role = match role_str.as_str() {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        "tool" => MessageRole::Tool,
        other => return Err(StoreError::InvalidData(format!("Unknown role: {other}"))),
    };

    let parts: Vec<MessagePart> = serde_json::from_str(&parts_json)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| StoreError::InvalidData(format!("Invalid created_at: {e}")))?
        .with_timezone(&Utc);

    Ok(Message {
        id,
        conversation_id,
        run_id,
        role,
        parts,
        created_at,
    })
}

fn extract_text_content(parts: &[MessagePart]) -> String {
    let mut texts = Vec::new();
    for part in parts {
        if let MessagePart::Text(text) = part {
            texts.push(text.as_str());
        }
    }
    texts.join(" ")
}

fn format_fts_query(query: &str) -> String {
    let words: Vec<String> = query
        .split_whitespace()
        .map(|w| {
            let sanitized = w.replace('"', "\"\"");
            format!("\"{sanitized}\"")
        })
        .collect();

    if words.is_empty() {
        "\"\"".to_string()
    } else {
        words.join(" AND ")
    }
}
