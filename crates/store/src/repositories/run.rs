use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::errors::StoreError;
use hubbub_domain::entities::run::{Run, RunStatus, Step, StepKind, StepStatus, UsageInfo};

pub async fn create_run(pool: &SqlitePool, run: &Run) -> Result<(), StoreError> {
    let id_str = run.id.to_string();
    let conv_id_str = run.conversation_id.to_string();
    let parent_run_id_str = run.parent_run_id.map(|p| p.to_string());
    let status_str = run_status_to_str(&run.status);
    let usage_json = match &run.usage {
        Some(u) => Some(serde_json::to_string(u)?),
        None => None,
    };
    let started_at_str = run.started_at.to_rfc3339();
    let finished_at_str = run.finished_at.map(|f| f.to_rfc3339());

    sqlx::query(
        r#"
        INSERT INTO runs (id, conversation_id, agent_id, parent_run_id, status, usage_json, cost_usd, started_at, finished_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id_str)
    .bind(conv_id_str)
    .bind(&run.agent_id)
    .bind(parent_run_id_str)
    .bind(status_str)
    .bind(usage_json)
    .bind(run.cost_usd)
    .bind(started_at_str)
    .bind(finished_at_str)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_run(pool: &SqlitePool, run: &Run) -> Result<(), StoreError> {
    let id_str = run.id.to_string();
    let status_str = run_status_to_str(&run.status);
    let usage_json = match &run.usage {
        Some(u) => Some(serde_json::to_string(u)?),
        None => None,
    };
    let finished_at_str = run.finished_at.map(|f| f.to_rfc3339());

    sqlx::query(
        r#"
        UPDATE runs
        SET status = ?, usage_json = ?, cost_usd = ?, finished_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status_str)
    .bind(usage_json)
    .bind(run.cost_usd)
    .bind(finished_at_str)
    .bind(id_str)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn fail_interrupted_runs(pool: &SqlitePool) -> Result<u64, StoreError> {
    let result = sqlx::query(
        r#"
        UPDATE runs
        SET status = ?, finished_at = ?
        WHERE status IN (?, ?)
        "#,
    )
    .bind(run_status_to_str(&RunStatus::Failed))
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(run_status_to_str(&RunStatus::Queued))
    .bind(run_status_to_str(&RunStatus::Running))
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn get_run(pool: &SqlitePool, id: Uuid) -> Result<Option<Run>, StoreError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        r#"
        SELECT id, conversation_id, agent_id, parent_run_id, status, usage_json, cost_usd, started_at, finished_at
        FROM runs
        WHERE id = ?
        "#,
    )
    .bind(id_str)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let id: String = row.try_get("id")?;
            let conv_id: String = row.try_get("conversation_id")?;
            let parent_run_id_str: Option<String> = row.try_get("parent_run_id")?;
            let status_str: String = row.try_get("status")?;
            let usage_json: Option<String> = row.try_get("usage_json")?;
            let cost_usd: Option<f64> = row.try_get("cost_usd")?;
            let started_at_str: String = row.try_get("started_at")?;
            let finished_at_str: Option<String> = row.try_get("finished_at")?;

            let parsed_id = Uuid::parse_str(&id)
                .map_err(|e| StoreError::InvalidData(format!("Invalid UUID: {e}")))?;
            let conversation_id = Uuid::parse_str(&conv_id).map_err(|e| {
                StoreError::InvalidData(format!("Invalid conversation_id UUID: {e}"))
            })?;
            let parent_run_id = match parent_run_id_str {
                Some(s) => Some(Uuid::parse_str(&s).map_err(|e| {
                    StoreError::InvalidData(format!("Invalid parent_run_id UUID: {e}"))
                })?),
                None => None,
            };
            let status = str_to_run_status(&status_str)?;
            let usage: Option<UsageInfo> = match usage_json {
                Some(j) => Some(serde_json::from_str(&j)?),
                None => None,
            };
            let started_at = DateTime::parse_from_rfc3339(&started_at_str)
                .map_err(|e| StoreError::InvalidData(format!("Invalid started_at: {e}")))?
                .with_timezone(&Utc);
            let finished_at = match finished_at_str {
                Some(s) => Some(
                    DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| StoreError::InvalidData(format!("Invalid finished_at: {e}")))?
                        .with_timezone(&Utc),
                ),
                None => None,
            };

            let agent_id: String = row.try_get("agent_id")?;

            Ok(Some(Run {
                id: parsed_id,
                conversation_id,
                agent_id,
                parent_run_id,
                status,
                usage,
                cost_usd,
                started_at,
                finished_at,
            }))
        }
        None => Ok(None),
    }
}

pub async fn create_step(pool: &SqlitePool, step: &Step) -> Result<(), StoreError> {
    let id_str = step.id.to_string();
    let run_id_str = step.run_id.to_string();
    let kind_str = match step.kind {
        StepKind::Llm => "llm",
        StepKind::Tool => "tool",
        StepKind::Approval => "approval",
    };
    let input_json = serde_json::to_string(&step.input)?;
    let output_json = match &step.output {
        Some(o) => Some(serde_json::to_string(o)?),
        None => None,
    };
    let status_str = match step.status {
        StepStatus::Running => "running",
        StepStatus::Completed => "completed",
        StepStatus::Failed => "failed",
        StepStatus::Cancelled => "cancelled",
    };
    let duration_ms_int = step.duration_ms.map(|d| d as i64);
    let created_at_str = step.created_at.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO steps (id, run_id, idx, kind, input_json, output_json, status, duration_ms, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            status = excluded.status,
            output_json = excluded.output_json,
            duration_ms = excluded.duration_ms
        "#,
    )
    .bind(id_str)
    .bind(run_id_str)
    .bind(step.idx as i64)
    .bind(kind_str)
    .bind(input_json)
    .bind(output_json)
    .bind(status_str)
    .bind(duration_ms_int)
    .bind(created_at_str)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_steps(pool: &SqlitePool, run_id: Uuid) -> Result<Vec<Step>, StoreError> {
    let run_id_str = run_id.to_string();
    let rows = sqlx::query(
        r#"
        SELECT id, run_id, idx, kind, input_json, output_json, status, duration_ms, created_at
        FROM steps
        WHERE run_id = ?
        ORDER BY idx ASC
        "#,
    )
    .bind(run_id_str)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let id_str: String = row.try_get("id")?;
        let run_id_str: String = row.try_get("run_id")?;
        let idx_int: i64 = row.try_get("idx")?;
        let kind_str: String = row.try_get("kind")?;
        let input_json: String = row.try_get("input_json")?;
        let output_json: Option<String> = row.try_get("output_json")?;
        let status_str: String = row.try_get("status")?;
        let duration_ms_int: Option<i64> = row.try_get("duration_ms")?;
        let created_at_str: String = row.try_get("created_at")?;

        let id = Uuid::parse_str(&id_str)
            .map_err(|e| StoreError::InvalidData(format!("Invalid UUID: {e}")))?;
        let run_id = Uuid::parse_str(&run_id_str)
            .map_err(|e| StoreError::InvalidData(format!("Invalid run_id UUID: {e}")))?;
        let kind = match kind_str.as_str() {
            "llm" => StepKind::Llm,
            "tool" => StepKind::Tool,
            "approval" => StepKind::Approval,
            other => {
                return Err(StoreError::InvalidData(format!(
                    "Unknown step kind: {other}"
                )));
            }
        };
        let input: serde_json::Value = serde_json::from_str(&input_json)?;
        let output: Option<serde_json::Value> = match output_json {
            Some(j) => Some(serde_json::from_str(&j)?),
            None => None,
        };
        let status = match status_str.as_str() {
            "running" => StepStatus::Running,
            "completed" => StepStatus::Completed,
            "failed" => StepStatus::Failed,
            "cancelled" => StepStatus::Cancelled,
            other => {
                return Err(StoreError::InvalidData(format!(
                    "Unknown step status: {other}"
                )));
            }
        };
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| StoreError::InvalidData(format!("Invalid created_at: {e}")))?
            .with_timezone(&Utc);

        result.push(Step {
            id,
            run_id,
            idx: idx_int as u32,
            kind,
            input,
            output,
            status,
            duration_ms: duration_ms_int.map(|d| d as u64),
            created_at,
        });
    }

    Ok(result)
}

fn run_status_to_str(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::Queued => "queued",
        RunStatus::Running => "running",
        RunStatus::WaitingApproval => "waiting_approval",
        RunStatus::Completed => "completed",
        RunStatus::Failed => "failed",
        RunStatus::Cancelled => "cancelled",
    }
}

fn str_to_run_status(s: &str) -> Result<RunStatus, StoreError> {
    match s {
        "queued" => Ok(RunStatus::Queued),
        "running" => Ok(RunStatus::Running),
        "waiting_approval" => Ok(RunStatus::WaitingApproval),
        "completed" => Ok(RunStatus::Completed),
        "failed" => Ok(RunStatus::Failed),
        "cancelled" => Ok(RunStatus::Cancelled),
        other => Err(StoreError::InvalidData(format!(
            "Unknown run status: {other}"
        ))),
    }
}
