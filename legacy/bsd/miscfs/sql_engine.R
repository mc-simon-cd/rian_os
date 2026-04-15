# -----------------------------------------------------------------------------
# Copyright 2026 simon_projec
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# -----------------------------------------------------------------------------

# modules/sql_engine.R
# R-SQL Engine: In-memory table operations

if (!exists("sys_sql")) {
  sys_sql <- new.env(parent = emptyenv())
  sys_sql$tables <- list()
}

sql_create <- function(table_name, schema) {
  if (!is.null(sys_sql$tables[[table_name]])) {
    return(sprintf("Table %s already exists.", table_name))
  }
  
  # schema format expected: "id:integer, name:character, age:integer"
  cols <- lapply(strsplit(schema, ",")[[1]], trimws)
  df_cols <- list()
  
  for (col in cols) {
    parts <- trimws(strsplit(col, ":")[[1]])
    if (length(parts) == 2) {
      if (parts[2] == "integer") df_cols[[parts[1]]] <- integer()
      else if (parts[2] == "character") df_cols[[parts[1]]] <- character()
      else if (parts[2] == "numeric") df_cols[[parts[1]]] <- numeric()
      else df_cols[[parts[1]]] <- character()
    }
  }
  
  sys_sql$tables[[table_name]] <- as.data.frame(df_cols, stringsAsFactors = FALSE)
  mmu_alloc(0, 10) # Allocate 10KB RAM (PID 0 for System)
  kernel_log("SQL", sprintf("Created table %s", table_name))
  return(sprintf("Created table %s", table_name))
}

sql_insert <- function(table_name, values) {
  if (is.null(sys_sql$tables[[table_name]])) {
    return(sprintf("Table %s does not exist.", table_name))
  }
  
  # values: list of values matching columns
  tryCatch({
    df <- sys_sql$tables[[table_name]]
    new_row <- as.data.frame(as.list(values), stringsAsFactors = FALSE)
    names(new_row) <- names(df)
    
    sys_sql$tables[[table_name]] <- rbind(df, new_row)
    kernel_log("SQL", sprintf("Inserted %d row into table %s", 1, table_name))
    return(sprintf("1 row inserted into %s", table_name))
  }, error = function(e) {
    return(sprintf("Insert error: %s", e$message))
  })
}

sql_select <- function(table_name, condition = NULL) {
  if (is.null(sys_sql$tables[[table_name]])) {
    return(sprintf("Table %s does not exist.", table_name))
  }
  
  df <- sys_sql$tables[[table_name]]
  
  if (!is.null(condition) && nchar(condition) > 0) {
    # Warning: Using eval(parse) is dangerous in real systems, but fine for simulation logic
    parsed_cond <- parse(text = condition)
    res <- tryCatch({
      subset(df, eval(parsed_cond, envir = df))
    }, error = function(e) df)
    return(res)
  } else {
    return(df)
  }
}
