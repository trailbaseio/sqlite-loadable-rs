use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, Result};

pub fn check_auxdata(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let label = api::value_text(values.get(0).unwrap()).unwrap();
    let value = api::value_text(values.get(1).unwrap()).unwrap();

    assert!(api::auxdata_get::<String>(context, 1).is_none());

    let b = Box::new(String::from(value));
    api::auxdata_set(context, 1, b);

    let entry = api::auxdata_get::<String>(context, 1).unwrap();
    assert!(entry == value);

    api::result_text(context, &format!("{label}={value}")).unwrap();

    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_test_auxdata_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(db, "check_auxdata", 2, check_auxdata, FunctionFlags::UTF8)?;
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
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_test_auxdata_init as *const (),
            )));
        }

        let conn = builder.connect().unwrap();

        // NOTE: even nested expressions are evaluated in different contexts leading to an
        // auxdata_get miss. auxdata_get/set is not suitable for naive caching across function
        // evaluations.
        let result: String = conn
            .prepare("SELECT (check_auxdata(?1, check_auxdata(?2, ?3)))")
            .await
            .unwrap()
            .query_row(["outer_label", "inner_label", "value"])
            .await
            .unwrap()
            .get(0)
            .unwrap();

        assert_eq!(result, "outer_label=inner_label=value");
    }
}
