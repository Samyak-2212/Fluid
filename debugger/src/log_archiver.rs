use std::fs;
use std::path::PathBuf;

pub fn archive_session(session_id: &str, bug_id: &str) -> std::io::Result<()> {
    let active_path = PathBuf::from(format!("debugger/logs/active/{}.log", session_id));
    
    let mut archive_dir = PathBuf::from("debugger/logs/archive");
    archive_dir.push(bug_id);
    
    fs::create_dir_all(&archive_dir)?;
    
    let mut archive_path = archive_dir.clone();
    archive_path.push(format!("{}.log", session_id));
    
    if active_path.exists() {
        fs::rename(active_path, archive_path)?;
    }
    
    Ok(())
}
