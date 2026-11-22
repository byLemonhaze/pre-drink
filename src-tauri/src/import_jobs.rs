//! Encrypted preparation checkpoints, followed by one atomic archive publication.
//! ZIPs/paths are never copied to sidecars. Incomplete work is absent from MCP/FTS.
use crate::{
    db::Vault,
    import::{export_conversations, import_warnings, parse_export_conversation, prepare_archive},
    model::{ImportReport, ParsedConversation},
    session::{Session, Store},
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::{
    fs::File,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
};

const PAUSED: &str =
    "Import paused. Your archive was not changed. Choose the ZIP again to continue.";

#[derive(Clone)]
pub(crate) struct ImportRun {
    cancelled: Arc<AtomicBool>,
    phase: Arc<AtomicU8>,
}
impl ImportRun {
    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::SeqCst) {
            Err(PAUSED.into())
        } else {
            Ok(())
        }
    }
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cancelled, &other.cancelled)
    }
}

#[derive(Serialize)]
pub struct ImportCheckpoint {
    pub label: String,
    pub saved: usize,
    pub total: usize,
}
#[derive(Serialize)]
pub struct ImportState {
    pub checkpoint: Option<ImportCheckpoint>,
    pub running: bool,
    pub phase: &'static str,
}

fn count(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<usize> {
    Ok(row.get::<_, u32>(index)? as usize)
}

impl Vault {
    pub fn import_checkpoint(&self) -> Result<Option<ImportCheckpoint>, String> {
        self.connection
            .query_row(
                "SELECT label,saved,total FROM import_job WHERE id=1",
                [],
                |r| {
                    Ok(ImportCheckpoint {
                        label: r.get(0)?,
                        saved: count(r, 1)?,
                        total: count(r, 2)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    fn begin_import(&self, fingerprint: &str, label: &str, total: usize) -> Result<usize, String> {
        let previous: Option<(String, usize, usize)> = self
            .connection
            .query_row(
                "SELECT fingerprint,saved,total FROM import_job WHERE id=1",
                [],
                |r| Ok((r.get(0)?, count(r, 1)?, count(r, 2)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some((old, saved, old_total)) = previous {
            if old != fingerprint || old_total != total {
                return Err("This ZIP does not match the saved import. Choose the original ZIP, or discard its saved preparation before starting another. Your archive was not changed.".into());
            }
            return Ok(saved);
        }
        self.connection
            .execute(
                "INSERT INTO import_job(id,fingerprint,label,total) VALUES(1,?1,?2,?3)",
                params![fingerprint, label, total as i64],
            )
            .map_err(|e| e.to_string())?;
        Ok(0)
    }

    fn save_import_batch(
        &mut self,
        expected: usize,
        items: &[ParsedConversation],
        skipped: usize,
    ) -> Result<(), String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let saved: usize = tx
            .query_row("SELECT saved FROM import_job WHERE id=1", [], |r| {
                count(r, 0)
            })
            .map_err(|_| "The saved import no longer exists.")?;
        if saved != expected {
            return Err("The import checkpoint changed. Resume it again.".into());
        }
        for (offset, item) in items.iter().enumerate() {
            let duplicate: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM import_items WHERE conversation_id=?1)",
                    [&item.id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if duplicate {
                return Err("The ZIP contains duplicate conversation IDs. Nothing was imported. Discard its preparation and choose a corrected export.".into());
            }
            let payload = serde_json::to_string(item)
                .map_err(|_| "Couldn't checkpoint this conversation.")?;
            tx.execute(
                "INSERT INTO import_items(position,conversation_id,payload) VALUES(?1,?2,?3)",
                params![(expected + offset) as i64, item.id, payload],
            )
            .map_err(|e| e.to_string())?;
        }
        let messages: usize = items.iter().map(|c| c.messages.len()).sum();
        tx.execute("UPDATE import_job SET saved=saved+?1,messages=messages+?2,skipped=skipped+?3 WHERE id=1",params![items.len() as i64,messages as i64,skipped as i64]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }

    pub(crate) fn discard_import(&self) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM import_job WHERE id=1", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn ready_report(&self) -> Result<ImportReport, String> {
        let (saved, total, messages, skipped): (usize, usize, usize, usize) = self
            .connection
            .query_row(
                "SELECT saved,total,messages,skipped FROM import_job WHERE id=1",
                [],
                |r| Ok((count(r, 0)?, count(r, 1)?, count(r, 2)?, count(r, 3)?)),
            )
            .map_err(|_| "No saved import is ready.")?;
        if saved != total {
            return Err("Choose the original ZIP to finish preparing this import.".into());
        }
        let mut report = ImportReport {
            conversations: total,
            messages,
            skipped_messages: skipped,
            ..Default::default()
        };
        import_warnings(&mut report);
        Ok(report)
    }
    fn import_payload_page(&self, after: usize) -> Result<Vec<String>, String> {
        let mut stmt = self
            .connection
            .prepare(
                "SELECT payload FROM import_items WHERE position>=?1 ORDER BY position LIMIT 20",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([after as i64], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    fn publish_prepared(
        &mut self,
        items: &[ParsedConversation],
        report: ImportReport,
    ) -> Result<ImportReport, String> {
        let current = self.ready_report()?;
        if items.len() != current.conversations || current.conversations != report.conversations {
            return Err("Import checkpoint is incomplete. Your archive was not changed.".into());
        }
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let report = Self::import_transaction(&tx, items, report)?;
        tx.execute("DELETE FROM import_job WHERE id=1", [])
            .map_err(|e| e.to_string())?;
        #[cfg(test)]
        crate::storage_failure::import_checkpoint(&self.path, &tx);
        tx.commit().map_err(|e| e.to_string())?;
        Ok(report)
    }
    #[cfg(test)]
    fn publish_import(&mut self, run: &ImportRun) -> Result<ImportReport, String> {
        let report = self.ready_report()?;
        let mut items = Vec::new();
        while items.len() < report.conversations {
            let rows = self.import_payload_page(items.len())?;
            if rows.is_empty() {
                return Err("Missing checkpoint rows.".into());
            }
            for row in rows {
                items.push(decode_item(&row, run)?);
            }
        }
        self.publish_prepared(&items, report)
    }
}

fn decode_item(payload: &str, run: &ImportRun) -> Result<ParsedConversation, String> {
    run.check()?;
    let mut c: ParsedConversation = serde_json::from_str(payload).map_err(|_| {
        "Saved import is unreadable. Discard its preparation and choose the ZIP again."
    })?;
    // Recheck current privacy outside the session mutex: lock/pause remains responsive.
    c.sensitive |= crate::privacy::is_sensitive(&c.title);
    for m in &mut c.messages {
        run.check()?;
        m.sensitive |= crate::privacy::is_sensitive(&m.content);
        c.sensitive |= m.sensitive;
    }
    Ok(c)
}

impl Session {
    pub fn import_state(&mut self) -> Result<ImportState, String> {
        let checkpoint = self.access()?.import_checkpoint()?;
        Ok(ImportState {
            checkpoint,
            running: self.import_run.is_some(),
            phase: match self
                .import_run
                .as_ref()
                .map(|r| r.phase.load(Ordering::SeqCst))
            {
                Some(0) => "reading",
                Some(1) => "preparing",
                Some(_) => "saving",
                None => "paused",
            },
        })
    }
    pub fn pause_import(&mut self) -> Result<(), String> {
        self.access()?;
        if let Some(run) = &self.import_run {
            run.cancel();
        }
        Ok(())
    }
    pub fn discard_import(&mut self) -> Result<(), String> {
        self.access()?;
        if self.import_run.is_some() {
            return Err("Pause the import before discarding its preparation.".into());
        }
        self.access()?.discard_import()
    }
}

fn with_run<T>(
    store: &Store,
    generation: u64,
    run: &ImportRun,
    f: impl FnOnce(&mut Vault) -> Result<T, String>,
) -> Result<T, String> {
    run.check()?;
    let mut session = store.lock().map_err(|_| "Vault unavailable.")?;
    session.expire();
    if session.generation != generation || !session.import_run.as_ref().is_some_and(|r| r.same(run))
    {
        return Err(PAUSED.into());
    }
    run.check()?;
    f(session.access()?)
}

/// Called on a worker thread. None finishes already fully prepared encrypted work.
pub fn run_import(store: Store, path: Option<PathBuf>) -> Result<ImportReport, String> {
    run_import_with_progress(store, path, |_| {})
}

pub(crate) fn run_import_with_progress(
    store: Store,
    path: Option<PathBuf>,
    progress: impl Fn(&'static str),
) -> Result<ImportReport, String> {
    let (generation, run) = {
        let mut s = store.lock().map_err(|_| "Vault unavailable.")?;
        s.access()?;
        if s.import_run.is_some() {
            return Err("An import is already running. Pause it first.".into());
        }
        let run = ImportRun {
            cancelled: Arc::new(AtomicBool::new(false)),
            phase: Arc::new(AtomicU8::new(0)),
        };
        s.import_run = Some(run.clone());
        (s.generation, run)
    };
    progress("reading");
    let result = (|| {
        if let Some(path) = path {
            if !path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("zip"))
            {
                return Err("Choose an official export ZIP.".into());
            }
            let file = File::open(&path).map_err(|_| {
                "Couldn't open the selected ZIP. Choose its current location to resume."
            })?;
            let input = prepare_archive(file, || run.check())?;
            let label = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .chars()
                .filter(|c| !c.is_control())
                .take(120)
                .collect::<String>();
            let saved = with_run(&store, generation, &run, |v| {
                v.begin_import(&input.fingerprint, &label, input.total)
            })?;
            run.phase.store(1, Ordering::SeqCst);
            progress("preparing");
            let mut ordinal = 0;
            let mut batch = Vec::new();
            let mut report = ImportReport::default();
            let mut bytes = 0;
            for (data, grok) in &input.documents {
                for conversation in export_conversations(data, *grok)? {
                    run.check()?;
                    if ordinal >= saved {
                        let parsed = parse_export_conversation(conversation, *grok, &mut report)?;
                        #[cfg(test)]
                        tests::PARSED.with(|count| count.set(count.get() + 1));
                        bytes += parsed.raw_json.len();
                        batch.push(parsed);
                    }
                    ordinal += 1;
                    if batch.len() >= 20 || bytes >= 4 * 1024 * 1024 {
                        with_run(&store, generation, &run, |v| {
                            v.save_import_batch(
                                ordinal - batch.len(),
                                &batch,
                                report.skipped_messages,
                            )
                        })?;
                        #[cfg(test)]
                        tests::after_checkpoint(&store);
                        batch.clear();
                        report = ImportReport::default();
                        bytes = 0;
                    }
                }
            }
            if !batch.is_empty() {
                with_run(&store, generation, &run, |v| {
                    v.save_import_batch(ordinal - batch.len(), &batch, report.skipped_messages)
                })?;
            }
        }
        run.phase.store(1, Ordering::SeqCst);
        progress("preparing");
        let report = with_run(&store, generation, &run, |v| v.ready_report())?;
        let mut items = Vec::new();
        while items.len() < report.conversations {
            let rows = with_run(&store, generation, &run, |v| {
                v.import_payload_page(items.len())
            })?;
            if rows.is_empty() {
                return Err(
                    "Import checkpoint is incomplete. Your archive was not changed.".into(),
                );
            }
            for payload in rows {
                items.push(decode_item(&payload, &run)?);
            }
        }
        run.phase.store(2, Ordering::SeqCst);
        progress("saving");
        with_run(&store, generation, &run, |v| {
            v.publish_prepared(&items, report)
        })
    })();
    if let Ok(mut s) = store.lock() {
        if s.import_run.as_ref().is_some_and(|r| r.same(&run)) {
            s.import_run = None;
        }
    }
    result
}

#[cfg(test)]
#[path = "import_job_tests.rs"]
mod tests;
