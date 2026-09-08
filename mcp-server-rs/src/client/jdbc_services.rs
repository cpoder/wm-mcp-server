//! High-level JDBC adapter service builders: `jdbc_custom_sql_create` and
//! `jdbc_batch_insert_create`.
//!
//! Designer fills ~30 template properties from a few resource-domain
//! lookups; done by hand over `adapter_service_create` every one of them is
//! a chance to get a silent refusal from the Adapter Runtime. The property
//! sets below were verified end to end on IS 12.1 / JDBC Adapter 10.3 /
//! PostgreSQL (the Winfarm ETL proof of concept, 24 services, ~1M rows).

use super::adapters::adapter_values_to_array;
use serde_json::{Map, Value, json};

pub const CUSTOM_SQL_TEMPLATE: &str = "com.wm.adapter.wmjdbc.services.CustomSQL";
pub const BATCH_INSERT_TEMPLATE: &str = "com.wm.adapter.wmjdbc.services.BatchInsert";

/// One input parameter or output column of a CustomSQL service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlField {
    pub name: String,
    /// JDBC type name as the adapter spells it: BIGINT, VARCHAR, NUMERIC,
    /// TIMESTAMP, DATE, BOOLEAN, INTEGER, SMALLINT, ...
    pub jdbc_type: String,
    /// Java type of the pipeline field; `java.lang.String` works for every
    /// JDBC type in both directions (dates as `yyyy-MM-dd HH:mm:ss.SSS`).
    pub java_type: String,
}

impl SqlField {
    pub fn from_json(v: &Value, what: &str, i: usize) -> Result<Self, String> {
        let name = v
            .get("name")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("{what}[{i}].name is required"))?;
        let jdbc_type = v
            .get("jdbc_type")
            .or_else(|| v.get("jdbcType"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_uppercase();
        let java_type = v
            .get("java_type")
            .or_else(|| v.get("javaType"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("java.lang.String")
            .to_string();
        Ok(Self {
            name: name.to_string(),
            jdbc_type,
            java_type,
        })
    }
}

/// Standard properties every JDBC adapter service carries (Designer emits
/// them; they wire the optional `overrideCredentials` input).
fn common_settings() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("designTimeLocale".into(), json!("en"));
    m.insert("userid".into(), json!("overrideCredentials.$dbUser"));
    m.insert("useridType".into(), json!("java.lang.String"));
    m.insert(
        "inputUseridSign".into(),
        json!("overrideCredentials.$dbUser"),
    );
    m.insert("password".into(), json!("overrideCredentials.$dbPassword"));
    m.insert("passwordType".into(), json!("java.lang.String"));
    m.insert(
        "inputPasswordSign".into(),
        json!("overrideCredentials.$dbPassword"),
    );
    m
}

/// Number of `?` bind markers outside single-quoted literals.
pub fn count_placeholders(sql: &str) -> usize {
    let mut in_quote = false;
    let mut n = 0;
    for c in sql.chars() {
        match c {
            '\'' => in_quote = !in_quote,
            '?' if !in_quote => n += 1,
            _ => {}
        }
    }
    n
}

/// `true` for statements that produce no result set (INSERT / UPDATE /
/// DELETE / TRUNCATE / CALL / MERGE without RETURNING): when the adapter's
/// SQL analysis fails on them, an empty output list is the right answer, not
/// an error.
pub fn returns_no_rows(sql: &str) -> bool {
    let upper = sql.trim_start().to_uppercase();
    let first = upper
        .trim_start_matches(|c: char| c == '(' || c.is_whitespace())
        .split(|c: char| !c.is_ascii_alphabetic())
        .next()
        .unwrap_or("");
    matches!(
        first,
        "INSERT"
            | "UPDATE"
            | "DELETE"
            | "TRUNCATE"
            | "CALL"
            | "MERGE"
            | "CREATE"
            | "DROP"
            | "ALTER"
    ) && !upper.contains("RETURNING")
}

/// Parse the `customSQLcolInfo` domain value: one line per column,
/// `index;name;JDBCTYPE;IN|OUT;`, IN and OUT indexed separately. Returns
/// `(inputs, outputs)` as `(index, name, jdbc_type)`. `-1` means the
/// adapter's SQL parser (fdb-sql-parser) could not analyse the statement.
#[allow(clippy::type_complexity)]
pub fn parse_col_info(
    col_info: &str,
) -> Result<(Vec<(usize, String, String)>, Vec<(usize, String, String)>), String> {
    if col_info.trim() == "-1" {
        return Err("customSQLcolInfo returned -1: the adapter cannot parse this SQL (joins with aliases, subqueries, functions, || ...); pass inputs/outputs explicitly".into());
    }
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    for raw in col_info.replace("\\n", "\n").split('\n') {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() < 4 {
            return Err(format!(
                "unexpected customSQLcolInfo line {line:?} (want idx;name;TYPE;IN|OUT;)"
            ));
        }
        let idx: usize = parts[0]
            .trim()
            .parse()
            .map_err(|_| format!("bad column index in customSQLcolInfo line {line:?}"))?;
        let entry = (
            idx,
            parts[1].trim().to_string(),
            parts[2].trim().to_uppercase(),
        );
        match parts[3].trim() {
            "IN" => ins.push(entry),
            "OUT" => outs.push(entry),
            other => {
                return Err(format!(
                    "unexpected direction {other:?} in customSQLcolInfo line {line:?}"
                ));
            }
        }
    }
    Ok((ins, outs))
}

/// Build the `colInfo` property from explicit field lists (same format as
/// the lookup answer, IN and OUT indexed separately).
pub fn build_col_info(inputs: &[SqlField], outputs: &[SqlField]) -> String {
    let mut s = String::new();
    for (i, f) in inputs.iter().enumerate() {
        s.push_str(&format!("{i};{};{};IN;\n", f.name, f.jdbc_type));
    }
    for (i, f) in outputs.iter().enumerate() {
        s.push_str(&format!("{i};{};{};OUT;\n", f.name, f.jdbc_type));
    }
    s
}

/// Complete `adapterServiceSettings` for a CustomSQL service. Empty arrays
/// are fine here: `adapter_service_create` strips them before the call.
pub fn custom_sql_settings(
    sql: &str,
    inputs: &[SqlField],
    outputs: &[SqlField],
    result_row_field: Option<&str>,
    max_row: Option<&str>,
    query_timeout: Option<&str>,
) -> Value {
    let idx = |n: usize| (0..n).map(|i| i.to_string()).collect::<Vec<_>>();
    let mut m = common_settings();
    m.insert("sql".into(), json!(sql));
    m.insert("sqlFieldType".into(), json!("java.lang.String"));
    m.insert("colInfo".into(), json!(build_col_info(inputs, outputs)));
    m.insert("inputColIndexes".into(), json!(idx(inputs.len())));
    m.insert(
        "inputExpression".into(),
        json!(inputs.iter().map(|f| &f.name).collect::<Vec<_>>()),
    );
    m.insert(
        "inputJDBCType".into(),
        json!(inputs.iter().map(|f| &f.jdbc_type).collect::<Vec<_>>()),
    );
    m.insert(
        "inputFieldType".into(),
        json!(inputs.iter().map(|f| &f.java_type).collect::<Vec<_>>()),
    );
    m.insert(
        "inputField".into(),
        json!(inputs.iter().map(|f| &f.name).collect::<Vec<_>>()),
    );
    m.insert(
        "realInputFields".into(),
        json!(inputs.iter().map(|f| &f.name).collect::<Vec<_>>()),
    );
    m.insert("outputColIndexes".into(), json!(idx(outputs.len())));
    m.insert(
        "outputExpression".into(),
        json!(outputs.iter().map(|f| &f.name).collect::<Vec<_>>()),
    );
    m.insert(
        "outputJDBCType".into(),
        json!(outputs.iter().map(|f| &f.jdbc_type).collect::<Vec<_>>()),
    );
    m.insert(
        "outputFieldType".into(),
        json!(outputs.iter().map(|f| &f.java_type).collect::<Vec<_>>()),
    );
    m.insert(
        "outputField".into(),
        json!(outputs.iter().map(|f| &f.name).collect::<Vec<_>>()),
    );
    m.insert(
        "resultField".into(),
        json!(
            outputs
                .iter()
                .map(|f| format!("results[].{}", f.name))
                .collect::<Vec<_>>()
        ),
    );
    m.insert(
        "resultFieldType".into(),
        json!(
            outputs
                .iter()
                .map(|f| format!("{}[]", f.java_type))
                .collect::<Vec<_>>()
        ),
    );
    m.insert(
        "realOutputField".into(),
        json!(
            outputs
                .iter()
                .map(|f| format!("results[].{}", f.name))
                .collect::<Vec<_>>()
        ),
    );
    m.insert("maxRow".into(), json!(max_row.unwrap_or("0")));
    m.insert("queryTimeOut".into(), json!(query_timeout.unwrap_or("-1")));
    m.insert(
        "resultRowField".into(),
        json!(result_row_field.unwrap_or("")),
    );
    m.insert(
        "resultRowFieldType".into(),
        json!(if result_row_field.is_some_and(|s| !s.is_empty()) {
            "java.lang.String"
        } else {
            ""
        }),
    );
    Value::Object(m)
}

/// One column as described by the `columnInfo` resource domain
/// (`com.wm.adapter.wmjdbc.connection.ColumnDesc`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDesc {
    pub name: String,
    /// e.g. `CHARACTER(10) VARYING NOT NULL`, `integer NOT NULL`
    pub column_type: String,
    /// `java.sql.Types` code
    pub jdbc_code: i32,
    pub index: i32,
}

/// Decode a `columnInfo` value: an AdapterValues array of columns, each
/// column itself an AdapterValues array `[name, type, jdbcCode, index,
/// identifierQuote?]` (so the field separators arrive as the two characters
/// `\n` inside the column entry).
pub fn parse_column_info(column_info: &str) -> Vec<ColumnDesc> {
    adapter_values_to_array(column_info)
        .iter()
        .filter_map(|entry| {
            let parts = adapter_values_to_array(entry);
            let parts: Vec<&str> = if parts.len() >= 2 {
                parts.iter().map(String::as_str).collect()
            } else {
                entry.split('\n').collect()
            };
            if parts.len() < 2 {
                return None;
            }
            Some(ColumnDesc {
                name: parts[0].to_string(),
                column_type: parts[1].to_string(),
                jdbc_code: parts
                    .get(2)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(12),
                index: parts
                    .get(3)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0),
            })
        })
        .collect()
}

/// `java.sql.Types` code -> the JDBC type name the adapter uses in its
/// `*.JDBCType` properties.
pub fn jdbc_type_name(code: i32) -> &'static str {
    match code {
        -7 => "BIT",
        -6 => "TINYINT",
        5 => "SMALLINT",
        4 => "INTEGER",
        -5 => "BIGINT",
        6 => "FLOAT",
        7 => "REAL",
        8 => "DOUBLE",
        2 => "NUMERIC",
        3 => "DECIMAL",
        1 => "CHAR",
        12 => "VARCHAR",
        -1 => "LONGVARCHAR",
        91 => "DATE",
        92 => "TIME",
        93 => "TIMESTAMP",
        16 => "BOOLEAN",
        2005 => "CLOB",
        2004 => "BLOB",
        -2 => "BINARY",
        -3 => "VARBINARY",
        -4 => "LONGVARBINARY",
        -15 => "NCHAR",
        -9 => "NVARCHAR",
        -16 => "LONGNVARCHAR",
        2011 => "NCLOB",
        2009 => "SQLXML",
        1111 => "OTHER",
        _ => "VARCHAR",
    }
}

/// Complete `adapterServiceSettings` for a BatchInsert service on one table.
/// `columns` are `(name, column_type, jdbc_type_name)` in table order.
pub fn batch_insert_settings(
    catalog: &str,
    schema: &str,
    table: &str,
    column_info: &str,
    columns: &[(String, String, String)],
    query_timeout: Option<&str>,
) -> Value {
    let names: Vec<&str> = columns.iter().map(|c| c.0.as_str()).collect();
    let types: Vec<&str> = columns.iter().map(|c| c.1.as_str()).collect();
    let jdbc: Vec<&str> = columns.iter().map(|c| c.2.as_str()).collect();
    let n = columns.len();
    let mut m = common_settings();
    m.insert("tables.tableIndexes".into(), json!(["T1"]));
    m.insert("tables.catalogName".into(), json!([catalog]));
    m.insert("tables.schemaName".into(), json!([schema]));
    m.insert("tables.tableName".into(), json!([table]));
    m.insert("tables.tableType".into(), json!(["TABLE"]));
    m.insert("tables.columnInfo".into(), json!([column_info]));
    m.insert("tables.realSchemaName".into(), json!([schema]));
    m.insert("update.column".into(), json!(names));
    m.insert("update.columnType".into(), json!(types));
    m.insert("update.JDBCType".into(), json!(jdbc));
    m.insert("update.expression".into(), json!(vec!["?"; n]));
    m.insert("update.inputColumn".into(), json!(names));
    m.insert("update.inputColumnType".into(), json!(types));
    m.insert("update.inputJDBCType".into(), json!(jdbc));
    m.insert("update.inputField".into(), json!(names));
    m.insert(
        "update.inputFieldType".into(),
        json!(vec!["java.lang.String"; n]),
    );
    m.insert(
        "update.batchInputField".into(),
        json!(
            names
                .iter()
                .map(|c| format!("inputs[].{c}"))
                .collect::<Vec<_>>()
        ),
    );
    m.insert(
        "update.batchInputFieldType".into(),
        json!(vec!["java.lang.String[]"; n]),
    );
    m.insert(
        "update.realInputField".into(),
        json!(
            names
                .iter()
                .map(|c| format!("inputs[].{c}"))
                .collect::<Vec<_>>()
        ),
    );
    m.insert(
        "update.queryTimeOut".into(),
        json!(query_timeout.unwrap_or("-1")),
    );
    m.insert("updatecount.fieldName".into(), json!("updateCount"));
    m.insert(
        "updatecount.updateCountOutputName".into(),
        json!(["updateCount[]"]),
    );
    m.insert(
        "updatecount.updateCountOutputType".into(),
        json!(["java.lang.String[]"]),
    );
    m.insert("updatecount.realOutput".into(), json!(["updateCount[]"]));
    Value::Object(m)
}

fn short_name(service_name: &str) -> &str {
    service_name.rsplit(':').next().unwrap_or(service_name)
}

impl super::ISClient {
    /// Create a CustomSQL adapter service from the SQL text and its
    /// parameters/columns. Output columns are taken from the adapter's own
    /// SQL analysis (`customSQLcolInfo`) when not given; input parameters
    /// must be named by the caller (one per `?`, in order).
    #[allow(clippy::too_many_arguments)]
    pub async fn jdbc_custom_sql_create(
        &self,
        service_name: &str,
        package_name: &str,
        connection_alias: &str,
        sql: &str,
        inputs: Option<Vec<SqlField>>,
        outputs: Option<Vec<SqlField>>,
        result_row_field: Option<&str>,
        max_row: Option<&str>,
        query_timeout: Option<&str>,
    ) -> Result<Value, String> {
        let placeholders = count_placeholders(sql);
        let mut inputs = match inputs {
            Some(v) => v,
            None if placeholders == 0 => Vec::new(),
            None => {
                return Err(format!(
                    "the SQL has {placeholders} '?' placeholder(s): pass inputs=[{{\"name\": ..., \"jdbc_type\": ...}}, ...] in bind order (names become the fields of {}Input)",
                    short_name(service_name)
                ));
            }
        };
        if inputs.len() != placeholders {
            return Err(format!(
                "{} input(s) given but the SQL has {placeholders} '?' placeholder(s); they must match one to one, in order",
                inputs.len()
            ));
        }

        // Ask the adapter to analyse the statement when something is missing.
        let need_lookup = outputs.is_none() || inputs.iter().any(|f| f.jdbc_type.is_empty());
        let mut lookup_note = None;
        let outputs = if need_lookup {
            let names = self
                .lookup_names(
                    connection_alias,
                    CUSTOM_SQL_TEMPLATE,
                    "customSQLcolInfo",
                    Some(&json!([sql])),
                )
                .await
                .map_err(|e| format!("customSQLcolInfo lookup failed: {e}"))?;
            let col_info = names.into_iter().next().unwrap_or_default();
            let (ins, outs) = match parse_col_info(&col_info) {
                Ok(parsed) => parsed,
                Err(_)
                    if returns_no_rows(sql)
                        && outputs.as_ref().is_none_or(Vec::is_empty)
                        && inputs.iter().all(|f| !f.jdbc_type.is_empty()) =>
                {
                    // INSERT/UPDATE/DELETE with typed inputs: nothing to learn
                    // from the analysis, no result columns to declare.
                    lookup_note = Some(
                        "the adapter could not analyse the statement (-1); accepted because it returns no rows and every input is typed",
                    );
                    (Vec::new(), Vec::new())
                }
                Err(e) => {
                    return Err(format!(
                        "{e} -- pass outputs=[{{\"name\": ..., \"jdbc_type\": ...}}] (an empty [] for INSERT/UPDATE/DELETE) and a jdbc_type on every input. The analysis also trips on schema-qualified names such as public.tag and on reserved words used as identifiers."
                    ));
                }
            };
            // Untyped inputs: by name when the adapter knows the column,
            // by bind position otherwise.
            for (i, f) in inputs.iter_mut().enumerate() {
                if f.jdbc_type.is_empty() {
                    f.jdbc_type = ins
                        .iter()
                        .find(|(_, n, _)| n == &f.name)
                        .or_else(|| ins.get(i))
                        .map(|(_, _, t)| t.clone())
                        .ok_or_else(|| format!("no jdbc_type for input {:?} and the adapter did not report parameter {i}", f.name))?;
                }
            }
            if outputs.is_none() && lookup_note.is_none() {
                lookup_note =
                    Some("output columns taken from the adapter's customSQLcolInfo analysis");
            }
            match outputs {
                Some(o) => o,
                None => outs
                    .into_iter()
                    .map(|(_, name, jdbc_type)| SqlField {
                        name,
                        jdbc_type,
                        java_type: "java.lang.String".into(),
                    })
                    .collect(),
            }
        } else {
            outputs.unwrap_or_default()
        };

        let settings = custom_sql_settings(
            sql,
            &inputs,
            &outputs,
            result_row_field,
            max_row,
            query_timeout,
        );
        let mut created = self
            .adapter_service_create(
                service_name,
                package_name,
                connection_alias,
                CUSTOM_SQL_TEMPLATE,
                Some(&settings),
            )
            .await?;
        let short = short_name(service_name);
        created["template"] = json!(CUSTOM_SQL_TEMPLATE);
        created["signature"] = json!({
            "input": format!("{short}Input/{{{}}}", inputs.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ")),
            "output": format!("{short}Output/results[]/{{{}}}{}", outputs.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
                result_row_field.filter(|s| !s.is_empty()).map(|r| format!(" + {short}Output/{r} (row count)")).unwrap_or_default()),
        });
        created["inputs"] = json!(
            inputs
                .iter()
                .map(
                    |f| json!({"name": f.name, "jdbc_type": f.jdbc_type, "java_type": f.java_type})
                )
                .collect::<Vec<_>>()
        );
        created["outputs"] = json!(
            outputs
                .iter()
                .map(
                    |f| json!({"name": f.name, "jdbc_type": f.jdbc_type, "java_type": f.java_type})
                )
                .collect::<Vec<_>>()
        );
        if let Some(n) = lookup_note {
            created["note"] = json!(n);
        }
        created["invoke_example"] = json!({
            "service_path": service_name,
            "inputs": {format!("{short}Input"): inputs.iter().map(|f| (f.name.clone(), json!("..."))).collect::<Map<_, _>>()},
        });
        Ok(created)
    }

    /// Create a BatchInsert adapter service for one table: columns and types
    /// come from the connection's `columnInfo` lookup, JDBC type names from
    /// the adapter's `updateJDBCTypes` lookup (falling back to the
    /// `java.sql.Types` code table).
    #[allow(clippy::too_many_arguments)]
    pub async fn jdbc_batch_insert_create(
        &self,
        service_name: &str,
        package_name: &str,
        connection_alias: &str,
        catalog: Option<&str>,
        schema: &str,
        table: &str,
        exclude_columns: &[String],
        include_columns: Option<&[String]>,
        query_timeout: Option<&str>,
    ) -> Result<Value, String> {
        // Default catalog: the adapter's own `<current catalog>` entry (what
        // Designer selects by default -- the generated INSERT then carries no
        // catalog qualifier and runs in the connection's database; verified
        // on PostgreSQL), otherwise the first catalog the driver lists.
        let catalog = match catalog.filter(|c| !c.is_empty()) {
            Some(c) => c.to_string(),
            None => {
                let names = self
                    .lookup_names(
                        connection_alias,
                        BATCH_INSERT_TEMPLATE,
                        "catalogNames",
                        None,
                    )
                    .await
                    .map_err(|e| format!("catalogNames lookup failed: {e}"))?;
                names
                    .iter()
                    .find(|n| n.as_str() == "<current catalog>")
                    .or_else(|| names.first())
                    .cloned()
                    .ok_or("the connection reports no catalog; pass catalog explicitly")?
            }
        };
        let column_info = self
            .lookup_names(
                connection_alias,
                BATCH_INSERT_TEMPLATE,
                "columnInfo",
                Some(&json!([catalog, schema, table])),
            )
            .await
            .map_err(|e| format!("columnInfo lookup failed: {e}"))?
            .into_iter()
            .next()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                format!("no columns found for {catalog}.{schema}.{table} (check tableNames lookup)")
            })?;
        let all = parse_column_info(&column_info);
        if all.is_empty() {
            return Err(format!(
                "could not parse columnInfo for {schema}.{table}: {column_info:?}"
            ));
        }

        // JDBC type names from the adapter itself (its own driver mapping),
        // static java.sql.Types table otherwise.
        let jdbc_names: Vec<String> = match self
            .lookup_domains(
                connection_alias,
                BATCH_INSERT_TEMPLATE,
                "updateJDBCTypes",
                Some(&json!([[column_info.clone()]])),
            )
            .await
        {
            Ok(domains) => domains
                .get("updateJDBCTypes")
                .filter(|v| v.len() == all.len())
                .cloned()
                .unwrap_or_else(|| {
                    all.iter()
                        .map(|c| jdbc_type_name(c.jdbc_code).to_string())
                        .collect()
                }),
            Err(_) => all
                .iter()
                .map(|c| jdbc_type_name(c.jdbc_code).to_string())
                .collect(),
        };

        let columns: Vec<(String, String, String)> = all
            .iter()
            .zip(jdbc_names)
            .filter(|(c, _)| {
                !exclude_columns
                    .iter()
                    .any(|x| x.eq_ignore_ascii_case(&c.name))
            })
            .filter(|(c, _)| {
                include_columns
                    .is_none_or(|inc| inc.iter().any(|x| x.eq_ignore_ascii_case(&c.name)))
            })
            .map(|(c, j)| (c.name.clone(), c.column_type.clone(), j))
            .collect();
        if columns.is_empty() {
            return Err(format!(
                "no column left after include/exclude for {schema}.{table} (table has: {})",
                all.iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let settings = batch_insert_settings(
            &catalog,
            schema,
            table,
            &column_info,
            &columns,
            query_timeout,
        );
        let mut created = self
            .adapter_service_create(
                service_name,
                package_name,
                connection_alias,
                BATCH_INSERT_TEMPLATE,
                Some(&settings),
            )
            .await?;
        let short = short_name(service_name);
        created["template"] = json!(BATCH_INSERT_TEMPLATE);
        created["catalog"] = json!(catalog);
        created["table"] = json!(format!("{catalog}.{schema}.{table}"));
        created["columns"] = json!(
            columns
                .iter()
                .map(|(n, t, j)| json!({"name": n, "column_type": t, "jdbc_type": j}))
                .collect::<Vec<_>>()
        );
        created["excluded"] = json!(
            all.iter()
                .filter(|c| !columns.iter().any(|(n, _, _)| n == &c.name))
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
        );
        created["signature"] = json!({
            "input": format!("{short}Input/inputs[] (document list, one document per row, fields: {})", columns.iter().map(|c| c.0.as_str()).collect::<Vec<_>>().join(", ")),
            "output": format!("{short}Output/updateCount[]"),
        });
        created["note"] = json!(
            "send inputs as a document LIST even for one row; a missing inputs list makes the driver fail with 'Invalid parameter binding(s)'"
        );
        Ok(created)
    }

    /// All resource domains returned by one lookup, by domain name.
    pub(crate) async fn lookup_domains(
        &self,
        connection_alias: &str,
        service_template: &str,
        resource_domain_name: &str,
        values: Option<&Value>,
    ) -> Result<std::collections::BTreeMap<String, Vec<String>>, String> {
        let v = self
            .adapter_resource_domain_lookup(
                connection_alias,
                service_template,
                resource_domain_name,
                values,
            )
            .await?;
        let mut out = std::collections::BTreeMap::new();
        for d in v
            .get("resourceDomainValues")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let name = d
                .get("resourceDomainName")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let vals = d
                .get("values")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.get("name").and_then(Value::as_str))
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            out.insert(name, vals);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::super::adapters::adapter_values_from_array;
    use super::*;

    fn f(name: &str, t: &str) -> SqlField {
        SqlField {
            name: name.into(),
            jdbc_type: t.into(),
            java_type: "java.lang.String".into(),
        }
    }

    #[test]
    fn row_less_statements_are_recognised() {
        assert!(returns_no_rows("DELETE FROM public.tag WHERE id = ?"));
        assert!(returns_no_rows("  update t set a = ? where b = ?"));
        assert!(returns_no_rows("INSERT INTO t (a) VALUES (?)"));
        assert!(!returns_no_rows(
            "INSERT INTO t (a) VALUES (?) RETURNING id"
        ));
        assert!(!returns_no_rows("SELECT dwh.truncate_star() AS result"));
        assert!(!returns_no_rows("WITH x AS (SELECT 1) SELECT * FROM x"));
    }

    #[test]
    fn placeholders_ignore_quoted_literals() {
        assert_eq!(count_placeholders("SELECT 1"), 0);
        assert_eq!(count_placeholders("WHERE a > ? AND b <= ?"), 2);
        assert_eq!(count_placeholders("WHERE k = 'why?' AND v = ?"), 1);
    }

    #[test]
    fn col_info_round_trip() {
        let ins = vec![f("from_id", "BIGINT"), f("to_id", "BIGINT")];
        let outs = vec![f("order_line_id", "BIGINT"), f("order_id", "VARCHAR")];
        let text = build_col_info(&ins, &outs);
        assert_eq!(
            text,
            "0;from_id;BIGINT;IN;\n1;to_id;BIGINT;IN;\n0;order_line_id;BIGINT;OUT;\n1;order_id;VARCHAR;OUT;\n"
        );
        let (i, o) = parse_col_info(&text).unwrap();
        assert_eq!(
            i,
            vec![
                (0, "from_id".into(), "BIGINT".into()),
                (1, "to_id".into(), "BIGINT".into())
            ]
        );
        assert_eq!(o[1], (1, "order_id".into(), "VARCHAR".into()));
        assert!(
            parse_col_info("-1")
                .unwrap_err()
                .contains("cannot parse this SQL")
        );
    }

    #[test]
    fn custom_sql_settings_match_the_verified_shape() {
        let s = custom_sql_settings(
            "SELECT a FROM t WHERE id > ?",
            &[f("id", "BIGINT")],
            &[f("a", "VARCHAR")],
            Some("rowCount"),
            None,
            None,
        );
        assert_eq!(s["colInfo"], "0;id;BIGINT;IN;\n0;a;VARCHAR;OUT;\n");
        assert_eq!(s["inputColIndexes"], json!(["0"]));
        assert_eq!(s["realInputFields"], json!(["id"]));
        assert_eq!(s["resultField"], json!(["results[].a"]));
        assert_eq!(s["resultFieldType"], json!(["java.lang.String[]"]));
        assert_eq!(s["maxRow"], "0");
        assert_eq!(s["queryTimeOut"], "-1");
        assert_eq!(s["resultRowField"], "rowCount");
        assert_eq!(s["resultRowFieldType"], "java.lang.String");
        assert_eq!(s["userid"], "overrideCredentials.$dbUser");
        // no parameters: arrays are empty (stripped later), resultRowField blank
        let s = custom_sql_settings("SELECT 1 AS x", &[], &[f("x", "INTEGER")], None, None, None);
        assert_eq!(s["inputField"], json!([]));
        assert_eq!(s["resultRowFieldType"], "");
    }

    #[test]
    fn column_info_is_decoded_from_the_double_adapter_values_encoding() {
        // Two columns as the columnInfo lookup returns them: fields inside a
        // column joined by escaped newlines, columns joined by real newlines.
        let col_a = adapter_values_from_array(&[
            "customer_code".into(),
            "CHARACTER(10) VARYING NOT NULL".into(),
            "12".into(),
            "1".into(),
            "\"".into(),
        ]);
        let col_b = adapter_values_from_array(&[
            "created".into(),
            "timestamp".into(),
            "93".into(),
            "2".into(),
            "\"".into(),
        ]);
        let column_info = adapter_values_from_array(&[col_a, col_b]);
        let cols = parse_column_info(&column_info);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].name, "customer_code");
        assert_eq!(cols[0].column_type, "CHARACTER(10) VARYING NOT NULL");
        assert_eq!(cols[0].jdbc_code, 12);
        assert_eq!(cols[1].jdbc_code, 93);
        assert_eq!(jdbc_type_name(cols[1].jdbc_code), "TIMESTAMP");
        assert_eq!(jdbc_type_name(-5), "BIGINT");
        assert_eq!(jdbc_type_name(424242), "VARCHAR");
    }

    #[test]
    fn batch_insert_settings_match_the_verified_shape() {
        let cols = vec![
            (
                "customer_code".to_string(),
                "CHARACTER(10) VARYING NOT NULL".to_string(),
                "VARCHAR".to_string(),
            ),
            (
                "customer_name".to_string(),
                "CHARACTER(80) VARYING".to_string(),
                "VARCHAR".to_string(),
            ),
        ];
        let s = batch_insert_settings("winfarm", "dwh", "dim_customer", "ci", &cols, None);
        assert_eq!(s["tables.tableIndexes"], json!(["T1"]));
        assert_eq!(s["tables.catalogName"], json!(["winfarm"]));
        assert_eq!(
            s["update.column"],
            json!(["customer_code", "customer_name"])
        );
        assert_eq!(s["update.expression"], json!(["?", "?"]));
        assert_eq!(
            s["update.batchInputField"],
            json!(["inputs[].customer_code", "inputs[].customer_name"])
        );
        assert_eq!(
            s["update.batchInputFieldType"],
            json!(["java.lang.String[]", "java.lang.String[]"])
        );
        assert_eq!(
            s["updatecount.updateCountOutputName"],
            json!(["updateCount[]"])
        );
        assert_eq!(s["update.queryTimeOut"], "-1");
    }

    #[test]
    fn sql_field_json_accepts_both_spellings_and_defaults_java_type() {
        let a = SqlField::from_json(&json!({"name": "id", "jdbc_type": "bigint"}), "inputs", 0)
            .unwrap();
        assert_eq!(a, f("id", "BIGINT"));
        let b = SqlField::from_json(
            &json!({"name": "d", "jdbcType": "DATE", "javaType": "java.sql.Date"}),
            "inputs",
            1,
        )
        .unwrap();
        assert_eq!(b.java_type, "java.sql.Date");
        assert!(
            SqlField::from_json(&json!({"jdbc_type": "DATE"}), "outputs", 2)
                .unwrap_err()
                .contains("outputs[2].name")
        );
    }
}
