use std::fs;
use crate::core::errors::BBResult;
use std::path::Path;

pub fn run(project_dir: &Path, confirm: bool) -> BBResult<()> {
    if !confirm {
        println!("Warning: This will permanently delete the .bb/ directory and all its contents.");
        println!("Run with --confirm to proceed.");
        return Ok(());
    }

    let bb_dir = project_dir.join(".bb");
    
    if !bb_dir.exists() {
        println!("No .bb/ directory found at {}", project_dir.display());
        return Ok(());
    }

    fs::remove_dir_all(&bb_dir)?;
    println!("Destroyed blackboard at {}/.bb/", project_dir.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::cli::commands::init;

    #[test]
    fn test_destroy_without_confirm() {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();
        
        // Without confirm, should not delete
        run(temp.path(), false).unwrap();
        assert!(temp.path().join(".bb").exists());
    }

    #[test]
    fn test_destroy_with_confirm() {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();
        
        // With confirm, should delete
        run(temp.path(), true).unwrap();
        assert!(!temp.path().join(".bb").exists());
    }
}
