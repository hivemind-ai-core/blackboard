use std::path::{Path, PathBuf};

pub fn find_blackboard_dir(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);

    while let Some(dir) = current {
        let bb_dir = dir.join(".bb");
        if bb_dir.is_dir() {
            return Some(bb_dir);
        }

        current = dir.parent();
    }

    None
}

pub fn is_initialized(project_dir: &Path) -> bool {
    let bb_dir = project_dir.join(".bb");
    let db_path = bb_dir.join("blackboard.db");

    bb_dir.is_dir() && db_path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_blackboard_dir_finds_bb() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();

        let found = find_blackboard_dir(temp.path());
        assert_eq!(found, Some(bb_dir));
    }

    #[test]
    fn test_find_blackboard_dir_from_subdir() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();

        let subdir = temp.path().join("src").join("components");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_blackboard_dir(&subdir);
        assert_eq!(found, Some(bb_dir));
    }

    #[test]
    fn test_find_blackboard_dir_not_found() {
        let temp = TempDir::new().unwrap();

        let found = find_blackboard_dir(temp.path());
        assert_eq!(found, None);
    }

    #[test]
    fn test_is_initialized_true() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();
        fs::write(bb_dir.join("blackboard.db"), "").unwrap();

        assert!(is_initialized(temp.path()));
    }

    #[test]
    fn test_is_initialized_false_no_bb_dir() {
        let temp = TempDir::new().unwrap();

        assert!(!is_initialized(temp.path()));
    }

    #[test]
    fn test_is_initialized_false_no_db() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();

        assert!(!is_initialized(temp.path()));
    }
}
