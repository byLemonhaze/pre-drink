use crate::{
    model::{ImportReport, Message, ParsedConversation},
    privacy::is_sensitive,
};
use serde_json::{Map, Value};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Seek},
    path::Path,
};

// Read JSON entries directly. Never extract archive paths onto the filesystem.
const MAX_JSON_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

pub fn read_chatgpt_zip(path: &Path) -> Result<(Vec<ParsedConversation>, ImportReport), String> {
    if !path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("zip"))
    {
        return Err("Choose an official ChatGPT export ZIP.".into());
    }
    let file = File::open(path)
        .map_err(|_| "Couldn't open the selected ZIP. Check its location and permissions.")?;
    read_archive(file)
}

pub fn read_archive<R: Read + Seek>(
    reader: R,
) -> Result<(Vec<ParsedConversation>, ImportReport), String> {
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|_| "This file isn't a readable ZIP archive.")?;
    if archive.len() > MAX_ENTRIES {
        return Err("This ZIP contains too many files.".into());
    }
    let mut entries = Vec::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| "Couldn't read a ZIP entry.")?;
        let name = entry
            .name()
            .map_err(|_| "A ZIP entry has an invalid filename.")?
            .replace('\\', "/");
        let base = name.rsplit('/').next().unwrap_or("");
        let numbered = base
            .strip_prefix("conversations-")
            .and_then(|s| s.strip_suffix(".json"))
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()));
        if base == "conversations.json" || numbered {
            total = total.saturating_add(entry.size());
            if total > MAX_JSON_BYTES {
                return Err("The conversation JSON exceeds the 256 MB import limit.".into());
            }
            entries.push(index);
        }
    }
    if entries.is_empty() {
        return Err("No conversations.json found. Import the original ChatGPT export ZIP. Claude and Grok imports are coming next.".into());
    }
    let mut result = Vec::new();
    let mut report = ImportReport::default();
    let mut seen = HashSet::new();
    let mut read_bytes = 0;
    for index in entries {
        let mut bytes = Vec::new();
        archive
            .by_index(index)
            .map_err(|_| "Couldn't read conversation data.")?
            .take(MAX_JSON_BYTES - read_bytes + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Couldn't decompress conversation data.")?;
        read_bytes += bytes.len() as u64;
        if read_bytes > MAX_JSON_BYTES {
            return Err("The conversation JSON exceeds the 256 MB import limit.".into());
        }
        let data: Value = serde_json::from_slice(&bytes)
            .map_err(|_| "The export contains invalid conversation JSON. Nothing was imported.")?;
        let conversations = data
            .as_array()
            .ok_or("Expected the ChatGPT conversations array. Nothing was imported.")?;
        for conversation in conversations {
            let parsed = parse_conversation(conversation, &mut report)?;
            if !seen.insert(parsed.id.clone()) {
                return Err(
                    "The ZIP contains duplicate conversation IDs. Nothing was imported.".into(),
                );
            }
            report.messages += parsed.messages.len();
            result.push(parsed);
        }
    }
    report.conversations = result.len();
    if report.skipped_messages > 0 {
        report.warnings.push(format!("{} hidden, tool, system, or non-text messages were not added to the reader. Original JSON is retained locally; attachments are not imported.", report.skipped_messages));
    }
    Ok((result, report))
}

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn time(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|n| n.is_finite())
}

fn content_text(content: &Value) -> String {
    if let Some(parts) = content.get("parts").and_then(Value::as_array) {
        parts
            .iter()
            .filter_map(|p| p.as_str().or_else(|| p.get("text").and_then(Value::as_str)))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        str_field(content, "text")
            .or_else(|| str_field(content, "content"))
            .unwrap_or("")
            .to_string()
    }
}

pub fn parse_conversation(
    value: &Value,
    report: &mut ImportReport,
) -> Result<ParsedConversation, String> {
    let source_id = str_field(value, "conversation_id")
        .or_else(|| str_field(value, "id"))
        .ok_or("A conversation is missing its ID. Nothing was imported.")?;
    // Length prefixes keep provider IDs containing punctuation unambiguous.
    let id = format!("chatgpt:{}:{}", source_id.len(), source_id);
    let node_id = |node: &str| format!("{}:{}", id, node);
    let mapping: &Map<String, Value> = value.get("mapping").and_then(Value::as_object).ok_or(
        "This isn't a supported ChatGPT export: missing message mapping. Nothing was imported.",
    )?;
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut roots = Vec::new();
    for (key, node) in mapping {
        if !node.is_object() {
            return Err(
                "A conversation contains an invalid message node. Nothing was imported.".into(),
            );
        }
        if let Some(parent) = str_field(node, "parent").filter(|p| mapping.contains_key(*p)) {
            children.entry(parent).or_default().push(key);
        } else {
            roots.push(key.as_str());
        }
    }
    let sort_nodes = |nodes: &mut Vec<&str>| {
        nodes.sort_by(|a, b| {
            let a_time = time(&mapping[*a]["message"], "create_time").unwrap_or(0.0);
            let b_time = time(&mapping[*b]["message"], "create_time").unwrap_or(0.0);
            a_time.total_cmp(&b_time).then_with(|| a.cmp(b))
        });
    };
    sort_nodes(&mut roots);
    for list in children.values_mut() {
        sort_nodes(list);
    }
    let current = str_field(value, "current_node").filter(|id| mapping.contains_key(*id));
    let fallback = mapping
        .keys()
        .filter(|id| !children.contains_key(id.as_str()))
        .max_by(|a, b| {
            time(&mapping[*a]["message"], "create_time")
                .unwrap_or(0.0)
                .total_cmp(&time(&mapping[*b]["message"], "create_time").unwrap_or(0.0))
                .then_with(|| a.cmp(b))
        })
        .map(String::as_str);
    if current.is_none() && !mapping.is_empty() && report.warnings.len() < 10 {
        report.warnings.push("A conversation has no valid current branch; the latest leaf is shown as its main branch.".into());
    }
    let mut active = HashSet::new();
    let mut cursor = current.or(fallback);
    while let Some(key) = cursor {
        if !active.insert(key) {
            return Err("A conversation has a cyclic message tree. Nothing was imported.".into());
        }
        cursor = mapping
            .get(key)
            .and_then(|n| str_field(n, "parent"))
            .filter(|p| mapping.contains_key(*p));
    }
    // Iterative traversal avoids stack exhaustion on very long conversations.
    let mut stack: Vec<_> = roots
        .iter()
        .rev()
        .map(|key| (*key, None::<String>, None::<String>))
        .collect();
    let mut visited = HashSet::new();
    let mut messages = Vec::new();
    while let Some((key, parent_id, inherited_branch)) = stack.pop() {
        if !visited.insert(key) {
            return Err("Invalid message tree. Nothing was imported.".into());
        }
        let node = &mapping[key];
        let branch = if active.contains(key) {
            None
        } else {
            inherited_branch.or_else(|| Some(node_id(key)))
        };
        let mut nearest_parent = parent_id.clone();
        let message = &node["message"];
        if !message.is_null() {
            let role = message["author"]["role"].as_str().unwrap_or("");
            let content = content_text(&message["content"]);
            let hidden = message["metadata"]["is_visually_hidden_from_conversation"]
                .as_bool()
                .unwrap_or(false);
            let recipient = str_field(message, "recipient").unwrap_or("all");
            let channel = str_field(message, "channel").unwrap_or("final");
            if matches!(role, "user" | "assistant")
                && !hidden
                && recipient == "all"
                && !matches!(channel, "analysis" | "commentary")
                && !content.trim().is_empty()
            {
                let message_id = node_id(key);
                nearest_parent = Some(message_id.clone());
                messages.push(Message {
                    id: message_id,
                    conversation_id: id.clone(),
                    role: role.into(),
                    sensitive: is_sensitive(&content),
                    content,
                    timestamp: time(message, "create_time"),
                    branch_id: branch.clone(),
                    parent_id,
                    is_active: active.contains(key),
                    position: messages.len() as i64,
                });
            } else {
                report.skipped_messages += 1;
            }
        }
        if let Some(list) = children.get(key) {
            for child in list.iter().rev() {
                stack.push((child, nearest_parent.clone(), branch.clone()));
            }
        }
    }
    if visited.len() != mapping.len() {
        return Err("A conversation has a cyclic message tree. Nothing was imported.".into());
    }
    let title = str_field(value, "title")
        .unwrap_or("Untitled conversation")
        .to_owned();
    Ok(ParsedConversation {
        sensitive: is_sensitive(&title) || messages.iter().any(|m| m.sensitive),
        id,
        title,
        created_at: time(value, "create_time"),
        updated_at: time(value, "update_time"),
        messages,
        raw_json: value.to_string(),
    })
}
