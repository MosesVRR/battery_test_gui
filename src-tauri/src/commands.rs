use tauri::command;

/// tauri command that calls the backend (rust) export_to_csv function
#[command]
pub fn export_csv_command(csv_path: String) -> Result<String, String> {
    let conn = crate::database::initialize_database().map_err(|e| format!("Failed to initialize database: {}", e))?;
    crate::file::export_to_csv(&conn, &csv_path).map_err(|err| err.to_string())?;
    Ok("CSV export successful".to_string())
}

///tauri command to get the project directory path or a parent directory's path (unused)
#[command]
pub fn get_project_dir(steps: usize) -> Result<String, String> {
    let current_dir = std::env::current_dir()
        .map_err(|err| err.to_string())?;
    
    let mut path = std::path::PathBuf::from(current_dir);
    
    for _ in 0..steps {
        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        } else {
            return Err("Reached the root directory. Cannot go up further.".to_string());
        }
    }
    
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert path to string.".to_string())
}