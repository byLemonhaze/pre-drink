//! Historical DDL with invented records, never a relabelled current vault.
use pre_drink::crypto;
use rusqlite::{types::Value, Connection};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

pub const PASSWORD: &str = "fictional historical garden phrase";
pub const TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub fn connect(path: &Path) -> Connection {
    let c = Connection::open(path).unwrap();
    crypto::configure(
        &c,
        &crypto::key(PASSWORD, &crypto::salt(path).unwrap()).unwrap(),
    )
    .unwrap();
    c
}

pub fn create(path: &Path, version: i64, upgraded_v5: bool) {
    assert!((2..=7).contains(&version));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).unwrap();
    let c = Connection::open(path).unwrap();
    crypto::configure(
        &c,
        &crypto::key(PASSWORD, &crypto::new_salt().unwrap()).unwrap(),
    )
    .unwrap();
    c.execute_batch(match version {
        2 => include_str!("../fixtures/migrations/historical/base-v2.sql"),
        3 => include_str!("../fixtures/migrations/historical/base-v3.sql"),
        _ => include_str!("../fixtures/migrations/historical/base-v4.sql"),
    })
    .unwrap();
    c.execute_batch("BEGIN;
      INSERT INTO conversations(id,platform,title,private,sensitive,raw_json) VALUES
      ('safe','chatgpt','Fictional cedar orchard',0,0,'{\"invented\":\"cedar\"}'),
      ('private','chatgpt','Fictional private orchard',1,0,'{}'),
      ('flagged','chatgpt','Fictional retained restriction',0,1,'{}'),
      ('newly-sensitive','chatgpt','Fictional old classifier',0,0,'{}');
      INSERT INTO messages(id,conversation_id,role,content,branch_id,parent_id,is_active,position) VALUES
      ('safe-root','safe','user','Cedar planters for the fictional orchard.','root',NULL,1,0),
      ('safe-alt','safe','assistant','Use clay pots on the alternate branch.','alt','safe-root',0,1),
      ('private-source','private','user','Keep the fictional orchard private.',NULL,NULL,1,0),
      ('old-secret','newly-sensitive','user','my wifi password is fictional-lilac',NULL,NULL,0,0);
      INSERT INTO memory(id,type,content,source_message_id,had_source,approved,created_at,updated_at) VALUES
      ('safe-note','project','Cedar orchard project.','safe-root',1,1,1,1),
      ('linked-secret','decision','A fictional garden decision.','old-secret',1,1,1,1),
      ('orphan','fact','An orphaned fictional note.',NULL,1,1,1,1);
      INSERT INTO briefs(id,title,content,approved,updated_at) VALUES
      ('safe-brief','Cedar plan','Use cedar planters.',1,1),
      ('linked-brief','Garden decision','Review the fictional garden decision.',1,1);
      INSERT INTO brief_sources(brief_id,memory_id,reviewed_at) VALUES
      ('safe-brief','safe-note',1),('linked-brief','linked-secret',1);
      INSERT INTO tags VALUES(1,'fictional garden');
      INSERT INTO conversation_tags VALUES('safe',1);
      INSERT INTO archive_snapshots VALUES('snapshot','safe','{\"invented\":\"earlier cedar\"}',1);
      INSERT INTO search_index(conversation_id,content) VALUES
      ('safe','Fictional cedar orchard Cedar planters for the fictional orchard. Use clay pots on the alternate branch.'),
      ('private','Fictional private orchard'),('flagged','Fictional retained restriction'),
      ('newly-sensitive','Fictional old classifier my wifi password is fictional-lilac');
      COMMIT;").unwrap();
    c.execute(
        "INSERT INTO agent_clients VALUES('client','Fictional historic curator',?1,1,1)",
        [hex::encode(Sha256::digest(TOKEN.as_bytes()))],
    )
    .unwrap();
    if version >= 4 {
        c.execute_batch(if version == 4 || upgraded_v5 {
            include_str!("../fixtures/migrations/historical/curation-v4.sql")
        } else if version == 7 {
            include_str!("../fixtures/migrations/historical/curation-v7.sql")
        } else {
            include_str!("../fixtures/migrations/historical/curation-v6.sql")
        })
        .unwrap();
        c.execute_batch("INSERT INTO agent_scopes VALUES('client','chatgpt',1);
          UPDATE curation_policy SET publish_session=1 WHERE provider='chatgpt';
          INSERT INTO curation_notes VALUES('held','chatgpt','project','Cedar orchard project.','archive','',1,1,0,0,1);
          INSERT INTO curation_sources VALUES('held','safe-root',1);
          INSERT INTO curation_revisions VALUES('held',1,'client','{\"fictional\":true}',1);
          INSERT INTO curation_requests VALUES('client','fictional-request','fictional-hash','{\"fictional\":true}');
          INSERT INTO curation_jobs VALUES('job','chatgpt','Cedar',1,'0','paused',1);
          INSERT INTO curation_job_sources VALUES('job',1,'safe-root',1);
          INSERT INTO curation_processed_sources VALUES('chatgpt','safe-alt',1);
          INSERT INTO curation_publications VALUES('held',1,'owner_review');").unwrap();
        if version == 7 {
            c.execute_batch("INSERT INTO curation_owner_holds VALUES('held');")
                .unwrap();
        }
        if upgraded_v5 {
            assert_eq!(version, 5);
            c.execute_batch("PRAGMA foreign_keys=OFF;").unwrap();
            c.execute_batch(include_str!(
                "../fixtures/migrations/historical/provider-v5.sql"
            ))
            .unwrap();
            c.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        }
    }
    c.pragma_update(None, "user_version", version).unwrap();
    c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
    assert_eq!(
        number(&c, "SELECT count(*) FROM pragma_foreign_key_check"),
        0
    );
}

pub fn number(c: &Connection, sql: &str) -> i64 {
    c.query_row(sql, [], |r| r.get(0)).unwrap()
}

pub fn rows(c: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut q = c.prepare(sql).unwrap();
    let n = q.column_count();
    q.query_map([], |r| {
        (0..n).map(|i| r.get(i)).collect::<Result<Vec<Value>, _>>()
    })
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap()
}

pub fn snapshot(c: &Connection) -> BTreeMap<String, Vec<Vec<Value>>> {
    let tables = rows(c,"SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_index_%' ORDER BY name");
    tables
        .into_iter()
        .map(|r| {
            let Value::Text(name) = &r[0] else {
                panic!("table name")
            };
            (
                name.clone(),
                rows(c, &format!("SELECT * FROM \"{name}\" ORDER BY rowid")),
            )
        })
        .collect()
}
