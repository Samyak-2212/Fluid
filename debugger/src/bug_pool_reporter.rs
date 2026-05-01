use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Review,
    Process,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Review => "review",
            Severity::Process => "process",
        }
    }
    
    pub fn heading(&self) -> &'static str {
        match self {
            Severity::Critical => "## Critical",
            Severity::High => "## High",
            Severity::Medium => "## Medium",
            Severity::Low => "## Low",
            Severity::Review => "## Pending Claude Review",
            Severity::Process => "## Process Violations",
        }
    }
}

pub struct BugEntry {
    pub id: String,
    pub severity: Severity,
    pub component: String,
    pub reported_by: String,
    pub description: String,
    pub reproduction: String,
}

pub fn report_bug(bug: BugEntry) -> std::io::Result<()> {
    let path = PathBuf::from("bug_pool/BUG_POOL.md");
    let content = fs::read_to_string(&path)?;
    
    let mut lines = content.lines().collect::<Vec<_>>();
    let mut insert_idx = None;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(bug.severity.heading()) {
            insert_idx = Some(i + 1);
            break;
        }
    }
    
    let idx = match insert_idx {
        Some(i) => {
            // Find next empty line to insert after heading
            let mut j = i;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            j
        },
        None => {
            // If heading not found, just append to the end.
            lines.push(bug.severity.heading());
            lines.len()
        }
    };
    
    let entry = format!(
        "\n### {}\n\
        - Severity: {}\n\
        - Component: {}\n\
        - Reported by: {}\n\
        - Description: {}\n\
        - Reproduction: {}\n\
        - Assigned to: UNASSIGNED\n\
        - Status: OPEN\n\
        - Resolution: \n",
        bug.id,
        bug.severity.as_str(),
        bug.component,
        bug.reported_by,
        bug.description,
        bug.reproduction
    );
    
    // We do atomic write: write to temp, rename
    let mut new_content = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i == idx {
            new_content.push_str(&entry);
        }
        new_content.push_str(line);
        new_content.push('\n');
    }
    if idx == lines.len() {
        new_content.push_str(&entry);
    }
    
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, new_content)?;
    fs::rename(temp_path, path)?;
    
    Ok(())
}
