// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::format;
use crate::libkern::dmesg::kernel_log;

pub struct SqlTable {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct SqlEngine {
    pub tables: BTreeMap<String, SqlTable>,
}

lazy_static::lazy_static! {
    pub static ref SYS_SQL: Mutex<SqlEngine> = Mutex::new(SqlEngine {
        tables: BTreeMap::new(),
    });
}

pub fn sql_create(table_name: &str, schema: &str) -> String {
    let mut state = SYS_SQL.lock();
    if state.tables.contains_key(table_name) {
        return format!("Table {} already exists.", table_name);
    }
    
    let columns: Vec<String> = schema.split(',').map(|s| s.trim().split(':').next().unwrap_or("").to_string()).collect();
    
    state.tables.insert(table_name.to_string(), SqlTable {
        name: table_name.to_string(),
        columns,
        rows: Vec::new(),
    });
    
    kernel_log("SQL", &format!("Created table {}", table_name));
    format!("Created table {}", table_name)
}

pub fn sql_insert(table_name: &str, values: Vec<String>) -> String {
    let mut state = SYS_SQL.lock();
    if let Some(table) = state.tables.get_mut(table_name) {
        table.rows.push(values);
        kernel_log("SQL", &format!("Inserted 1 row into table {}", table_name));
        format!("1 row inserted into {}", table_name)
    } else {
        format!("Table {} does not exist.", table_name)
    }
}

pub fn sql_select(table_name: &str) -> Result<Vec<Vec<String>>, String> {
    let state = SYS_SQL.lock();
    if let Some(table) = state.tables.get(table_name) {
        Ok(table.rows.clone())
    } else {
        Err(format!("Table {} does not exist.", table_name))
    }
}
