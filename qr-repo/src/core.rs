use rusqlite::{params, Connection, Result};

pub enum ColumnType {
    Integer,
    Text,
    Json,
    Bool,
}

pub struct Column {
    pub name: String,
    pub type_: ColumnType,
    pub required: bool,
    pub default_val: Option<String>,
}

impl Column {
    pub fn required(name: &str, type_: ColumnType) -> Self {
        Column {
            name: name.to_string(),
            type_,
            required: true,
            default_val: None,
        }
    }

    pub fn nullable(name: &str, type_: ColumnType) -> Self {
        Column {
            name: name.to_string(),
            type_,
            required: false,
            default_val: None,
        }
    }

    pub fn with_default(name: &str, type_: ColumnType, default_val: &str) -> Self {
        Column {
            name: name.to_string(),
            type_,
            required: false,
            default_val: Some(default_val.to_string()),
        }
    }
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

pub fn check_table_exist(conn: &Connection, table_name: &str, table_f: fn() -> Table) -> bool {
    let sql = "SELECT count(`name`) from `sqlite_master` WHERE `type` = 'table' AND name = ?";
    let mut statement = conn.prepare(sql).unwrap();
    let res = statement.query_row(params![table_name], |row| row.get(0) as Result<i32>);
    let exist = res.unwrap() > 0;
    if !exist {
        let table = table_f();
        match create_table(conn, table) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to create table: {:}", e)
            }
        };
    }
    exist
}

fn create_table(conn: &Connection, table: Table) -> Result<usize, rusqlite::Error> {
    let ddl: String = ddl(table);
    println!("{}", ddl);
    conn.execute(&ddl, params![])
}

fn col_ddl(column: &Column) -> String {
    let col_type = match column.type_ {
        ColumnType::Bool => "INTEGER",
        ColumnType::Integer => "INTEGER",
        ColumnType::Json => "TEXT",
        ColumnType::Text => "TEXT",
    };

    let mut ddl = format!("`{}`\t\t{}", column.name, col_type);
    if column.required {
        ddl.push_str(" NOT NULL");
    }
    match &column.default_val {
        Some(val) => ddl.push_str(&format!(" DEFAULT {}", val)),
        None => {}
    }
    if column.default_val.is_some() {};
    ddl
}

fn ddl(table: Table) -> String {
    let table_name: String = table.name;
    let list: Vec<String> = table.columns.iter().map(col_ddl).collect();
    let cols: String = list.join(",\n\t");

    let base: String = format!(
        "CREATE TABLE IF NOT EXISTS `{}` (\n\t{}, \n\t{}, \n\t{}, \n\t{}\n);",
        table_name,
        "`id`           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL",
        "`create_time`  TIMESTAMP NOT NULL DEFAULT current_timestamp",
        "`modify_time`  TIMESTAMP NOT NULL DEFAULT current_timestamp",
        cols,
    );
    base
}