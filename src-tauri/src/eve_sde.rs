//! Ingest and query EVE SDE `types.jsonl` / `groups.jsonl` stored in SQLite (`EveSdeTypes`, `EveSdeGroups`).

use crate::models::EveTypeSnapshot;
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;

use rusqlite::{params, params_from_iter, Connection};

fn html_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<[^>]*>").expect("tag strip regex"))
}

fn strip_html_tags(raw: &str) -> String {
    let s = html_tag_re().replace_all(raw, "").to_string();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn locale_en(obj: Option<&Value>) -> Option<String> {
    obj?.get("en")?.as_str().map(std::string::ToString::to_string)
}

fn type_value_to_snapshot(v: &Value) -> Result<EveTypeSnapshot, String> {
    let type_id = v
        .get("_key")
        .and_then(|x| x.as_i64())
        .ok_or_else(|| "type row missing _key".to_string())?;
    let name = locale_en(v.get("name"));
    let description = locale_en(v.get("description")).map(|s| strip_html_tags(&s));
    let base_price = v
        .get("basePrice")
        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)));
    let group_id = v.get("groupID").and_then(|x| x.as_i64());
    let volume = v
        .get("volume")
        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)));
    let published = v.get("published").and_then(|x| x.as_bool());
    let portion_size = v.get("portionSize").and_then(|x| x.as_i64());
    let mass = v
        .get("mass")
        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)));

    Ok(EveTypeSnapshot {
        type_id,
        name,
        description,
        base_price,
        group_id,
        group_name: None,
        volume,
        published,
        portion_size,
        mass,
    })
}

fn group_name_en_from_payload(payload: &str) -> Result<Option<String>, String> {
    let v: Value = serde_json::from_str(payload).map_err(|e| e.to_string())?;
    Ok(locale_en(v.get("name")))
}

/// Replace SDE tables with contents of the two JSONL files (full JSON line per row).
pub fn ingest_from_jsonl_paths(
    conn: &mut Connection,
    types_path: &Path,
    groups_path: &Path,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM EveSdeTypes", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM EveSdeGroups", [])
        .map_err(|e| e.to_string())?;
    insert_jsonl_into_table(&tx, "EveSdeTypes", "TypeId", types_path)?;
    insert_jsonl_into_table(&tx, "EveSdeGroups", "GroupId", groups_path)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn insert_jsonl_into_table(
    tx: &rusqlite::Transaction<'_>,
    table: &str,
    pk_col: &str,
    path: &Path,
) -> Result<(), String> {
    let f = File::open(path).map_err(|e| format!("open {path:?}: {e}"))?;
    let reader = BufReader::new(f);
    let sql = format!("INSERT INTO {table} ({pk_col}, Payload) VALUES (?1, ?2)");
    let mut stmt = tx.prepare(&sql).map_err(|e| e.to_string())?;
    for (line_no, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line).map_err(|e| {
            format!("{} line {}: {e}", path.display(), line_no + 1)
        })?;
        let key = v
            .get("_key")
            .and_then(|x| x.as_i64())
            .ok_or_else(|| format!("{} line {}: missing _key", path.display(), line_no + 1))?;
        stmt.execute(params![key, line]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn count_types(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM EveSdeTypes", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

fn fetch_group_names(
    conn: &Connection,
    group_ids: &HashSet<i64>,
) -> Result<HashMap<i64, Option<String>>, String> {
    let mut out: HashMap<i64, Option<String>> = HashMap::new();
    if group_ids.is_empty() {
        return Ok(out);
    }
    let list: Vec<i64> = group_ids.iter().copied().collect();
    let ph = (0..list.len()).map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT GroupId, Payload FROM EveSdeGroups WHERE GroupId IN ({ph})");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(list.iter().copied()), |row| {
            let gid: i64 = row.get(0)?;
            let payload: String = row.get(1)?;
            Ok((gid, payload))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (gid, payload) = row.map_err(|e| e.to_string())?;
        let name = group_name_en_from_payload(&payload).ok().flatten();
        out.insert(gid, name);
    }
    Ok(out)
}

/// Build [`EveTypeSnapshot`] for the given type ids (English strings, HTML stripped from descriptions).
pub fn lookup_types(conn: &Connection, type_ids: &[i64]) -> Result<HashMap<String, EveTypeSnapshot>, String> {
    if type_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let ph = (0..type_ids.len()).map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT TypeId, Payload FROM EveSdeTypes WHERE TypeId IN ({ph})");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(type_ids.iter().copied()), |row| {
            let id: i64 = row.get(0)?;
            let payload: String = row.get(1)?;
            Ok((id, payload))
        })
        .map_err(|e| e.to_string())?;

    let mut parsed: Vec<(i64, Value)> = Vec::new();
    let mut group_need: HashSet<i64> = HashSet::new();
    for row in rows {
        let (id, payload) = row.map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
        if let Some(g) = v.get("groupID").and_then(|x| x.as_i64()) {
            group_need.insert(g);
        }
        parsed.push((id, v));
    }

    let group_names = fetch_group_names(conn, &group_need)?;

    let mut out = HashMap::new();
    for (id, v) in parsed {
        let mut snap = type_value_to_snapshot(&v)?;
        if let Some(gid) = snap.group_id {
            snap.group_name = group_names.get(&gid).cloned().flatten();
        }
        out.insert(id.to_string(), snap);
    }
    Ok(out)
}
