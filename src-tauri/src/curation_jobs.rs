//! Durable, provider-bound source manifests. The external agent performs inference.
use crate::{
    curation::{hash, now, CurationOperation},
    db::Vault,
};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

impl Vault {
    pub(crate) fn save_curation_job(
        &self,
        provider: &str,
        client: &str,
        op: &CurationOperation,
    ) -> Result<Value, String> {
        let CurationOperation::SaveJob {
            id,
            expected_revision,
            label,
            checkpoint,
            status,
            request_key,
        } = op
        else {
            return Err("Invalid job request.".into());
        };
        if label.trim().is_empty()
            || label.len() > 200
            || crate::privacy::is_sensitive(label)
            || request_key.is_empty()
            || request_key.len() > 100
            || !matches!(
                status.as_str(),
                "running" | "paused" | "completed" | "cancelled"
            )
        {
            return Err("Use a short non-sensitive job label, retry key, and valid status.".into());
        }
        let request_hash = hash(op)?;
        let retry_key = format!("job:{request_key}");
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let previous:Option<(String,String)>=tx.query_row("SELECT request_hash,result FROM curation_requests WHERE client_id=?1 AND request_key=?2",params![client,retry_key],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        if let Some((h, result)) = previous {
            if h != request_hash {
                return Err("This retry key was already used for another job update.".into());
            }
            return serde_json::from_str(&result).map_err(|e| e.to_string());
        }
        let id = if let Some(id) = id {
            let previous:Option<(i64,String,String)>=tx.query_row("SELECT revision,checkpoint,status FROM curation_jobs WHERE id=?1 AND provider=?2",params![id,provider],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|e|e.to_string())?;
            let (revision, completed, state) = previous.ok_or("Job unavailable.")?;
            if revision != *expected_revision || matches!(state.as_str(), "completed" | "cancelled")
            {
                return Err("Job changed or finished. Refresh before retrying.".into());
            }
            let completed: u32 = completed.parse().map_err(|_| "Invalid saved checkpoint.")?;
            let count: u32 = tx
                .query_row(
                    "SELECT count(*) FROM curation_job_sources WHERE job_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if *checkpoint < completed
                || *checkpoint > count
                || (status == "completed" && *checkpoint != count)
            {
                return Err("Checkpoint must advance within the saved source manifest; finish all sources before completing.".into());
            }
            // Acknowledgement applies only to still-eligible, unchanged evidence. A
            // changed source remains pending for the next incremental job.
            tx.execute("INSERT INTO curation_processed_sources(provider,message_id,version) SELECT ?2,s.message_id,s.version FROM curation_job_sources s JOIN messages m ON m.id=s.message_id JOIN conversations c ON c.id=m.conversation_id JOIN source_versions v ON v.message_id=m.id WHERE s.job_id=?1 AND s.ordinal>?3 AND s.ordinal<=?4 AND c.platform=?2 AND c.private=0 AND c.sensitive=0 AND m.sensitive=0 AND v.version=s.version ON CONFLICT(provider,message_id) DO UPDATE SET version=excluded.version",params![id,provider,completed,checkpoint]).map_err(|e|e.to_string())?;
            tx.execute("UPDATE curation_jobs SET label=?1,checkpoint=?2,status=?3,revision=revision+1,updated_at=?4 WHERE id=?5",params![label,checkpoint.to_string(),status,now(),id]).map_err(|e|e.to_string())?;
            id.clone()
        } else {
            if *expected_revision != 0
                || *checkpoint != 0
                || !matches!(status.as_str(), "running" | "paused")
            {
                return Err(
                    "New jobs start at revision and checkpoint zero, running or paused.".into(),
                );
            }
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute("INSERT INTO curation_jobs(id,provider,label,checkpoint,status,updated_at) VALUES (?1,?2,?3,'0',?4,?5)",params![id,provider,label,status,now()]).map_err(|e|e.to_string())?;
            // Immutable IDs + evidence versions, not another copy of raw text.
            tx.execute("INSERT INTO curation_job_sources(job_id,ordinal,message_id,version) SELECT ?1,row_number() OVER (ORDER BY m.id),m.id,v.version FROM messages m JOIN conversations c ON c.id=m.conversation_id JOIN source_versions v ON v.message_id=m.id LEFT JOIN curation_processed_sources p ON p.provider=?2 AND p.message_id=m.id WHERE c.platform=?2 AND c.private=0 AND c.sensitive=0 AND m.sensitive=0 AND (p.version IS NULL OR p.version!=v.version)",params![id,provider]).map_err(|e|e.to_string())?;
            id
        };
        let count: i64 = tx
            .query_row(
                "SELECT count(*) FROM curation_job_sources WHERE job_id=?1",
                [&id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let revision = expected_revision
            .checked_add(1)
            .ok_or("Invalid revision.")?;
        let result =
            json!({"id":id,"revision":revision,"source_count":count,"checkpoint":checkpoint});
        tx.execute(
            "INSERT INTO curation_requests VALUES (?1,?2,?3,?4)",
            params![client, retry_key, request_hash, result.to_string()],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub(crate) fn curation_job_sources(
        &self,
        provider: &str,
        id: &str,
        after: u32,
    ) -> Result<Value, String> {
        let allowed: bool = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM curation_jobs WHERE id=?1 AND provider=?2)",
                params![id, provider],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !allowed {
            return Err("Job unavailable.".into());
        }
        let mut stmt=self.connection.prepare("SELECT s.ordinal,s.message_id,s.version,COALESCE(c.platform=?2 AND c.private=0 AND c.sensitive=0 AND m.sensitive=0 AND v.version=s.version,0) FROM curation_job_sources s LEFT JOIN messages m ON m.id=s.message_id LEFT JOIN conversations c ON c.id=m.conversation_id LEFT JOIN source_versions v ON v.message_id=m.id WHERE s.job_id=?1 AND s.ordinal>?3 ORDER BY s.ordinal LIMIT 30").map_err(|e|e.to_string())?;
        let rows=stmt.query_map(params![id,provider,after],|r|{let ordinal:i64=r.get(0)?;let available:bool=r.get(3)?;if available{Ok(json!({"ordinal":ordinal,"message_id":r.get::<_,String>(1)?,"version":r.get::<_,i64>(2)?,"available":true}))}else{Ok(json!({"ordinal":ordinal,"available":false}))}}).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        Ok(
            json!({"sources":rows,"next_after":rows.last().map(|r|&r["ordinal"]),"instructions":"Read each available source in chunks and confirm its version matches this manifest. Save contributions with retry keys before advancing checkpoint to the last completed ordinal. Skip unavailable entries; changed evidence will be eligible in a new incremental job."}),
        )
    }
}
