use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::calls::{CallEdge, Resolution};
use crate::refs::{RefKind, Reference};
use crate::symbol::{FileEntry, Import, SourceRange, Symbol, SymbolKind, Visibility};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    language TEXT NOT NULL,
    mtime INTEGER NOT NULL,
    hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    qualified_name TEXT NOT NULL,
    kind TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    start_col INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    end_col INTEGER NOT NULL,
    signature TEXT,
    visibility TEXT NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE IF NOT EXISTS imports (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    local_name TEXT NOT NULL,
    qualified_target TEXT NOT NULL,
    alias TEXT,
    start_line INTEGER NOT NULL,
    start_col INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE IF NOT EXISTS refs (
    id INTEGER PRIMARY KEY,
    target_symbol_id INTEGER,
    source_file_id INTEGER NOT NULL,
    source_symbol_id INTEGER,
    kind TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    start_col INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    end_col INTEGER NOT NULL,
    text TEXT NOT NULL,
    confidence REAL NOT NULL,
    FOREIGN KEY (target_symbol_id) REFERENCES symbols(id),
    FOREIGN KEY (source_file_id) REFERENCES files(id),
    FOREIGN KEY (source_symbol_id) REFERENCES symbols(id)
);

CREATE TABLE IF NOT EXISTS calls (
    id INTEGER PRIMARY KEY,
    caller_symbol_id INTEGER NOT NULL,
    callee_symbol_id INTEGER,
    callee_name TEXT,
    candidate_symbol_ids TEXT,
    ref_id INTEGER NOT NULL,
    confidence REAL NOT NULL,
    resolution TEXT NOT NULL,
    FOREIGN KEY (caller_symbol_id) REFERENCES symbols(id),
    FOREIGN KEY (callee_symbol_id) REFERENCES symbols(id),
    FOREIGN KEY (ref_id) REFERENCES refs(id)
);

CREATE INDEX IF NOT EXISTS idx_symbols_file_id ON symbols(file_id);
CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
CREATE INDEX IF NOT EXISTS idx_symbols_qualified_name ON symbols(qualified_name);
CREATE INDEX IF NOT EXISTS idx_imports_file_id ON imports(file_id);
CREATE INDEX IF NOT EXISTS idx_refs_target_symbol_id ON refs(target_symbol_id);
CREATE INDEX IF NOT EXISTS idx_refs_source_file_id ON refs(source_file_id);
CREATE INDEX IF NOT EXISTS idx_calls_caller_symbol_id ON calls(caller_symbol_id);
CREATE INDEX IF NOT EXISTS idx_calls_callee_symbol_id ON calls(callee_symbol_id);
"#;

pub struct Index {
    conn: Connection,
}

impl Index {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("failed to open index database")?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=OFF;",
        )
        .context("failed to set pragmas")?;
        Ok(Self { conn })
    }

    pub fn create(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let index = Self::open(path)?;
        index
            .conn
            .execute_batch(SCHEMA)
            .context("failed to create schema")?;
        Ok(index)
    }

    pub fn clear(&self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM calls; DELETE FROM refs; DELETE FROM imports; DELETE FROM symbols; DELETE FROM files;",
        )?;
        Ok(())
    }

    // --- Files ---

    pub fn insert_file(&self, path: &str, language: &str, mtime: i64, hash: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO files (path, language, mtime, hash) VALUES (?1, ?2, ?3, ?4)",
            params![path, language, mtime, hash],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn find_file(&self, path: &str) -> Result<Option<FileEntry>> {
        self.conn
            .query_row(
                "SELECT id, path, language, mtime, hash FROM files WHERE path = ?1",
                params![path],
                |row| {
                    Ok(FileEntry {
                        id: row.get(0)?,
                        path: row.get(1)?,
                        language: row.get(2)?,
                        mtime: row.get(3)?,
                        hash: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn find_file_by_id(&self, id: i64) -> Result<Option<FileEntry>> {
        self.conn
            .query_row(
                "SELECT id, path, language, mtime, hash FROM files WHERE id = ?1",
                params![id],
                |row| {
                    Ok(FileEntry {
                        id: row.get(0)?,
                        path: row.get(1)?,
                        language: row.get(2)?,
                        mtime: row.get(3)?,
                        hash: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn delete_file_data(&self, file_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM calls WHERE caller_symbol_id IN (SELECT id FROM symbols WHERE file_id = ?1)",
            params![file_id],
        )?;
        self.conn.execute(
            "DELETE FROM calls WHERE callee_symbol_id IN (SELECT id FROM symbols WHERE file_id = ?1)",
            params![file_id],
        )?;
        self.conn.execute(
            "DELETE FROM refs WHERE source_file_id = ?1",
            params![file_id],
        )?;
        self.conn
            .execute("DELETE FROM imports WHERE file_id = ?1", params![file_id])?;
        self.conn
            .execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id])?;
        self.conn
            .execute("DELETE FROM files WHERE id = ?1", params![file_id])?;
        Ok(())
    }

    // --- Symbols ---

    pub fn insert_symbol(&self, sym: &Symbol) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO symbols (file_id, name, qualified_name, kind, start_line, start_col, end_line, end_col, signature, visibility)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                sym.file_id,
                sym.name,
                sym.qualified_name,
                format!("{:?}", sym.kind).to_lowercase(),
                sym.range.start_line,
                sym.range.start_col,
                sym.range.end_line,
                sym.range.end_col,
                sym.signature,
                format!("{:?}", sym.visibility).to_lowercase(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn symbols_in_file(&self, file_id: i64) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, name, qualified_name, kind, start_line, start_col, end_line, end_col, signature, visibility
             FROM symbols WHERE file_id = ?1",
        )?;
        let rows = stmt.query_map(params![file_id], row_to_symbol)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_symbol_by_id(&self, id: i64) -> Result<Option<Symbol>> {
        self.conn
            .query_row(
                "SELECT id, file_id, name, qualified_name, kind, start_line, start_col, end_line, end_col, signature, visibility
                 FROM symbols WHERE id = ?1",
                params![id],
                row_to_symbol,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn find_symbols_by_name(&self, name: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, name, qualified_name, kind, start_line, start_col, end_line, end_col, signature, visibility
             FROM symbols WHERE name = ?1",
        )?;
        let rows = stmt.query_map(params![name], row_to_symbol)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_symbols_by_qualified_name(&self, qname: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, name, qualified_name, kind, start_line, start_col, end_line, end_col, signature, visibility
             FROM symbols WHERE qualified_name = ?1",
        )?;
        let rows = stmt.query_map(params![qname], row_to_symbol)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // --- Imports ---

    pub fn insert_import(&self, imp: &Import) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO imports (file_id, local_name, qualified_target, alias, start_line, start_col)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                imp.file_id,
                imp.local_name,
                imp.qualified_target,
                imp.alias,
                imp.start_line,
                imp.start_col,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn imports_in_file(&self, file_id: i64) -> Result<Vec<Import>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, local_name, qualified_target, alias, start_line, start_col
             FROM imports WHERE file_id = ?1",
        )?;
        let rows = stmt.query_map(params![file_id], |row| {
            Ok(Import {
                id: row.get(0)?,
                file_id: row.get(1)?,
                local_name: row.get(2)?,
                qualified_target: row.get(3)?,
                alias: row.get(4)?,
                start_line: row.get(5)?,
                start_col: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // --- References ---

    pub fn insert_ref(&self, r: &Reference) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO refs (target_symbol_id, source_file_id, source_symbol_id, kind, start_line, start_col, end_line, end_col, text, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                r.target_symbol_id,
                r.source_file_id,
                r.source_symbol_id,
                format!("{:?}", r.kind).to_lowercase(),
                r.start_line,
                r.start_col,
                r.end_line,
                r.end_col,
                r.text,
                r.confidence,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn find_ref_by_id(&self, id: i64) -> Result<Option<Reference>> {
        self.conn
            .query_row(
                "SELECT id, target_symbol_id, source_file_id, source_symbol_id, kind, start_line, start_col, end_line, end_col, text, confidence
                 FROM refs WHERE id = ?1",
                params![id],
                row_to_ref,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn refs_to_symbol(&self, target_symbol_id: i64) -> Result<Vec<Reference>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target_symbol_id, source_file_id, source_symbol_id, kind, start_line, start_col, end_line, end_col, text, confidence
             FROM refs WHERE target_symbol_id = ?1",
        )?;
        let rows = stmt.query_map(params![target_symbol_id], row_to_ref)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // --- Call Edges ---

    pub fn insert_call(&self, edge: &CallEdge) -> Result<i64> {
        let candidates = if edge.candidate_symbol_ids.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&edge.candidate_symbol_ids)?)
        };
        self.conn.execute(
            "INSERT INTO calls (caller_symbol_id, callee_symbol_id, callee_name, candidate_symbol_ids, ref_id, confidence, resolution)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                edge.caller_symbol_id,
                edge.callee_symbol_id,
                edge.callee_name,
                candidates,
                edge.ref_id,
                edge.confidence,
                format!("{:?}", edge.resolution).to_lowercase(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn call_edges_to(&self, callee_symbol_id: i64) -> Result<Vec<CallEdge>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, caller_symbol_id, callee_symbol_id, callee_name, candidate_symbol_ids, ref_id, confidence, resolution
             FROM calls WHERE callee_symbol_id = ?1",
        )?;
        let rows = stmt.query_map(params![callee_symbol_id], row_to_call)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn call_edges_from(&self, caller_symbol_id: i64) -> Result<Vec<CallEdge>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, caller_symbol_id, callee_symbol_id, callee_name, candidate_symbol_ids, ref_id, confidence, resolution
             FROM calls WHERE caller_symbol_id = ?1",
        )?;
        let rows = stmt.query_map(params![caller_symbol_id], row_to_call)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn transaction(&self) -> Result<()> {
        self.conn.execute_batch("BEGIN")?;
        Ok(())
    }

    pub fn commit(&self) -> Result<()> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }
}

fn parse_symbol_kind(s: &str) -> SymbolKind {
    match s {
        "function" => SymbolKind::Function,
        "method" => SymbolKind::Method,
        "traitmethod" => SymbolKind::TraitMethod,
        "traitmethoddecl" => SymbolKind::TraitMethodDecl,
        "classmethod" => SymbolKind::ClassMethod,
        "arrowfunction" => SymbolKind::ArrowFunction,
        "closure" => SymbolKind::Closure,
        _ => SymbolKind::Function,
    }
}

fn parse_visibility(s: &str) -> Visibility {
    match s {
        "public" => Visibility::Public,
        "pubcrate" => Visibility::PubCrate,
        "pubsuper" => Visibility::PubSuper,
        _ => Visibility::Private,
    }
}

fn parse_ref_kind(s: &str) -> RefKind {
    match s {
        "call" => RefKind::Call,
        "import" => RefKind::Import,
        "reexport" => RefKind::ReExport,
        "functionpointer" => RefKind::FunctionPointer,
        "traitref" => RefKind::TraitRef,
        "typeref" => RefKind::TypeRef,
        "test" => RefKind::Test,
        _ => RefKind::Call,
    }
}

fn parse_resolution(s: &str) -> Resolution {
    match s {
        "resolved" => Resolution::Resolved,
        "ambiguous" => Resolution::Ambiguous,
        _ => Resolution::Unresolved,
    }
}

fn row_to_symbol(row: &rusqlite::Row) -> rusqlite::Result<Symbol> {
    let kind_str: String = row.get(4)?;
    let vis_str: String = row.get(10)?;
    Ok(Symbol {
        id: row.get(0)?,
        file_id: row.get(1)?,
        name: row.get(2)?,
        qualified_name: row.get(3)?,
        kind: parse_symbol_kind(&kind_str),
        range: SourceRange {
            start_line: row.get(5)?,
            start_col: row.get(6)?,
            end_line: row.get(7)?,
            end_col: row.get(8)?,
        },
        signature: row.get(9)?,
        visibility: parse_visibility(&vis_str),
    })
}

fn row_to_ref(row: &rusqlite::Row) -> rusqlite::Result<Reference> {
    let kind_str: String = row.get(4)?;
    Ok(Reference {
        id: row.get(0)?,
        target_symbol_id: row.get(1)?,
        source_file_id: row.get(2)?,
        source_symbol_id: row.get(3)?,
        kind: parse_ref_kind(&kind_str),
        start_line: row.get(5)?,
        start_col: row.get(6)?,
        end_line: row.get(7)?,
        end_col: row.get(8)?,
        text: row.get(9)?,
        confidence: row.get(10)?,
    })
}

fn row_to_call(row: &rusqlite::Row) -> rusqlite::Result<CallEdge> {
    let candidates_str: Option<String> = row.get(4)?;
    let candidates = candidates_str
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let resolution_str: String = row.get(7)?;
    Ok(CallEdge {
        id: row.get(0)?,
        caller_symbol_id: row.get(1)?,
        callee_symbol_id: row.get(2)?,
        callee_name: row.get(3)?,
        candidate_symbol_ids: candidates,
        ref_id: row.get(5)?,
        confidence: row.get(6)?,
        resolution: parse_resolution(&resolution_str),
    })
}
