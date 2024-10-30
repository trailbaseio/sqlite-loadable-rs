//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api,
    scalar::scalar_function_raw,
    table::{
        define_table_function_with_find, BestIndexError, FindResult, IndexInfo, VTab,
        VTabArguments, VTabCursor, VTabFind,
    },
    Result,
};

use std::{mem, os::raw::c_int};

static CREATE_SQL: &str = "CREATE TABLE x(a)";
enum Columns {
    A,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::A),
        _ => None,
    }
}

#[repr(C)]
pub struct FindTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for FindTable {
    type Aux = ();
    type Cursor = FindCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, FindTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = FindTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut _info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        Ok(())
    }

    fn open(&mut self) -> Result<FindCursor> {
        Ok(FindCursor::new())
    }
}

impl<'vtab> VTabFind<'vtab> for FindTable {
    fn find_function(&mut self, _argc: i32, name: &str) -> Option<FindResult> {
        if name == "wrapped" {
            return Some((scalar_function_raw(wrapped), None, None));
        }
        None
    }
}

fn wrapped(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(
        context,
        format!("Wrapped access! {}", api::value_text(&values[0]).unwrap()),
    )?;
    Ok(())
}

#[repr(C)]
pub struct FindCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    value: Option<String>,
}
impl FindCursor {
    fn new() -> FindCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        FindCursor {
            base,
            rowid: 0,
            value: None,
        }
    }
}

impl VTabCursor for FindCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        self.rowid = 1;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.rowid >= 2
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::A) => api::result_text(context, "Bare A access!")?,
            _ => (),
        };
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}

#[sqlite_entrypoint]
pub fn sqlite3_find_init(db: *mut sqlite3) -> Result<()> {
    api::overload_function(db, "wrapped", 1)?;
    define_table_function_with_find::<FindTable>(db, "find", None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use libsql::ffi::sqlite3_auto_extension;
    use libsql::Builder;

    #[tokio::test]
    async fn test_libsql_auto_extension() {
        let builder = Builder::new_local(":memory:").build().await.unwrap();

        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_find_init as *const ())));
        }

        let db = builder.connect().unwrap();

        assert_eq!(
            db.prepare("select a from find")
                .await
                .unwrap()
                .query_row(())
                .await
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "Bare A access!"
        );
        assert_eq!(
            db.prepare("select wrapped(a) from find")
                .await
                .unwrap()
                .query_row(())
                .await
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "Wrapped access! Bare A access!"
        );
    }
}
