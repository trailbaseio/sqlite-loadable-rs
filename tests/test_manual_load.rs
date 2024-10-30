use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, Result};

pub fn add(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let a = api::value_int(&values[0]);
    let b = api::value_int(&values[1]);
    api::result_int(context, a + b);
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_manual_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(db, "addx", 2, add, FunctionFlags::empty())?;
    Ok(())
}

#[cfg(feature = "static")]
#[cfg(test)]
mod tests {
    use libsql::Builder;

    #[tokio::test]
    async fn test_manual_load() {
        let _db = Builder::new_local(":memory:")
            .build()
            .await
            .unwrap()
            .connect()
            .unwrap();

        // TODO when static linking is fixed, this should work
        // unsafe {
        //     sqlite3_manual_init(
        //         std::mem::transmute(db.handle()),
        //         std::ptr::null_mut(),
        //         std::ptr::null_mut(),
        //     );
        // }
        //
        // let result: i32 = db
        //     .prepare("select addx(?1, ?2)")
        //     .await
        //     .unwrap()
        //     .query_row([1, 2])
        //     .await
        //     .unwrap()
        //     .get(0)
        //     .unwrap();

        // assert_eq!(result, 3);
    }
}
