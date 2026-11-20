//! Independent adapters for official Claude/Grok account exports, not live capture.
use crate::{
    import::parse_tree,
    model::{ImportReport, ParsedConversation},
};
use serde_json::{json, Map, Value};
use std::collections::HashSet;

fn text<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

// ISO-8601 timestamps and Mongo extended-JSON milliseconds. Do not invent a
// current time for absent or unrecognized source timestamps.
fn timestamp(v: &Value) -> Option<f64> {
    if let Some(date) = v.get("$date") {
        if let Some(ms) = date.get("$numberLong") {
            return ms
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| ms.as_i64())
                .map(|n| n as f64 / 1000.0);
        }
        if let Some(ms) = date.as_i64() {
            return Some(ms as f64 / 1000.0);
        }
        return timestamp(date);
    }
    v.as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis() as f64 / 1000.0)
}

struct Entry<'a> {
    id: &'a str,
    parent: Option<&'a str>,
    role: &'a str,
    content: String,
    time: Option<f64>,
}

fn normalize(
    provider: &str,
    source_id: &str,
    title: &str,
    times: (Option<f64>, Option<f64>),
    entries: Vec<Entry<'_>>,
    raw: &Value,
    report: &mut ImportReport,
) -> Result<ParsedConversation, String> {
    let (created, updated) = times;
    let mut seen = HashSet::new();
    for e in &entries {
        if e.id.len() > 200 || !seen.insert(e.id) {
            return Err("Duplicate or invalid response ID. Nothing was imported.".into());
        }
    }
    // If the export contains no parent links, preserve its supplied linear order.
    // If it supplies a graph, keep that graph and let the shared parser reject cycles.
    let graph = entries.iter().any(|e| e.parent.is_some());
    let mut mapping = Map::new();
    let mut previous: Option<&str> = None;
    for e in &entries {
        let parent = if graph { e.parent } else { previous };
        mapping.insert(e.id.to_owned(), json!({"parent":parent,"message":{"author":{"role":e.role},"create_time":e.time,"content":{"parts":[e.content]}}}));
        previous = Some(e.id);
    }
    let normalized = json!({"id":source_id,"title":title,"create_time":created,"update_time":updated,"mapping":mapping,"current_node":if graph {None} else {previous}});
    let mut result = parse_tree(&normalized, report, provider)?;
    result.raw_json = raw.to_string();
    if graph && report.warnings.len() < 10 {
        report.warnings.push(format!("{provider}: parent links are preserved; the latest leaf is a display fallback, not a provider-confirmed active branch."));
    }
    Ok(result)
}

pub(crate) fn parse_grok(
    v: &Value,
    report: &mut ImportReport,
) -> Result<ParsedConversation, String> {
    let meta = v
        .get("conversation")
        .filter(|v| v.is_object())
        .ok_or("Missing Grok conversation metadata. Nothing was imported.")?;
    let id = text(meta, "id").ok_or("Missing Grok conversation ID. Nothing was imported.")?;
    let responses = v
        .get("responses")
        .and_then(Value::as_array)
        .ok_or("Missing Grok responses array. Nothing was imported.")?;
    let mut entries = Vec::new();
    for wrapper in responses {
        let r = wrapper.get("response").unwrap_or(wrapper);
        if !r.is_object() {
            return Err("Invalid Grok response. Nothing was imported.".into());
        }
        let id = text(r, "_id")
            .or_else(|| text(r, "response_id"))
            .ok_or("Missing Grok response ID. Nothing was imported.")?;
        let role = match text(r, "sender")
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "human" => "user",
            "assistant" => "assistant",
            _ => "omitted",
        };
        let content = match r.get("message") {
            Some(Value::String(s)) => s.clone(),
            None | Some(Value::Null) => String::new(),
            _ => return Err("Unsupported Grok message content. Nothing was imported.".into()),
        };
        entries.push(Entry {
            id,
            parent: text(r, "parent_response_id"),
            role,
            content,
            time: timestamp(&r["create_time"]),
        });
    }
    normalize(
        "grok",
        id,
        text(meta, "title").unwrap_or("Untitled Grok conversation"),
        (
            timestamp(&meta["create_time"]),
            timestamp(&meta["modify_time"]).or_else(|| timestamp(&meta["update_time"])),
        ),
        entries,
        v,
        report,
    )
}

pub(crate) fn parse_claude(
    v: &Value,
    report: &mut ImportReport,
) -> Result<ParsedConversation, String> {
    let id = text(v, "uuid").ok_or("Missing Claude conversation UUID. Nothing was imported.")?;
    let messages = v
        .get("chat_messages")
        .and_then(Value::as_array)
        .ok_or("Missing Claude messages array. Nothing was imported.")?;
    let mut entries = Vec::new();
    for m in messages {
        let id = text(m, "uuid").ok_or("Missing Claude message UUID. Nothing was imported.")?;
        let role = match text(m, "sender") {
            Some("human") => "user",
            Some("assistant") => "assistant",
            _ => "omitted",
        };
        let content = if let Some(blocks) = m.get("content").filter(|v| !v.is_null()) {
            let blocks = blocks
                .as_array()
                .ok_or("Unsupported Claude content blocks. Nothing was imported.")?;
            let mut texts = Vec::new();
            for block in blocks {
                if text(block, "type") == Some("text") {
                    texts.push(
                        block
                            .get("text")
                            .and_then(Value::as_str)
                            .ok_or("Invalid Claude text block. Nothing was imported.")?,
                    );
                }
            }
            if blocks.is_empty() {
                text(m, "text").unwrap_or("").to_owned()
            } else {
                texts.join("\n")
            }
        } else {
            text(m, "text").unwrap_or("").to_owned()
        };
        let parent =
            text(m, "parent_message_uuid").filter(|p| *p != "00000000-0000-0000-0000-000000000000");
        entries.push(Entry {
            id,
            parent,
            role,
            content,
            time: timestamp(&m["created_at"]),
        });
    }
    normalize(
        "claude",
        id,
        text(v, "name").unwrap_or("Untitled Claude conversation"),
        (timestamp(&v["created_at"]), timestamp(&v["updated_at"])),
        entries,
        v,
        report,
    )
}
