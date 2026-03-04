use std::path::Path;

use anyhow::Result;

use crate::index::{count_bean_formats, ArchiveIndex, Index};

/// Force rebuild index unconditionally from YAML files
pub fn cmd_sync(beans_dir: &Path) -> Result<()> {
    // Check for mixed formats before building
    let (md_count, yaml_count) = count_bean_formats(beans_dir)?;

    let index = Index::build(beans_dir)?;
    let count = index.beans.len();
    index.save(beans_dir)?;

    // Rebuild archive index
    let archive_index = ArchiveIndex::build(beans_dir)?;
    let archive_count = archive_index.beans.len();
    if archive_count > 0 || beans_dir.join("archive.yaml").exists() {
        archive_index.save(beans_dir)?;
    }

    println!("Index rebuilt: {} beans indexed.", count);
    if archive_count > 0 {
        println!(
            "Archive index rebuilt: {} archived beans indexed.",
            archive_count
        );
    }

    // Warn about mixed formats
    if md_count > 0 && yaml_count > 0 {
        eprintln!();
        eprintln!("Warning: Mixed bean formats detected!");
        eprintln!("  {} .md files (current format)", md_count);
        eprintln!("  {} .yaml files (legacy format)", yaml_count);
        eprintln!();
        eprintln!("This can cause confusion. Consider migrating legacy files:");
        eprintln!("  - Remove or archive .yaml files: mkdir -p .beans/legacy && mv .beans/*.yaml .beans/legacy/");
        eprintln!("  - Or run 'bn doctor' for more details");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bean::Bean;
    use crate::util::title_to_slug;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn sync_rebuilds_index() {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        let bean1 = Bean::new("1", "Task one");
        let bean2 = Bean::new("2", "Task two");

        let slug1 = title_to_slug(&bean1.title);
        let slug2 = title_to_slug(&bean2.title);

        bean1
            .to_file(beans_dir.join(format!("1-{}.md", slug1)))
            .unwrap();
        bean2
            .to_file(beans_dir.join(format!("2-{}.md", slug2)))
            .unwrap();

        // Sync should create index with 2 beans
        let result = cmd_sync(&beans_dir);
        assert!(result.is_ok());

        // Verify index was created
        assert!(beans_dir.join("index.yaml").exists());

        // Verify index contains both beans
        let index = Index::load(&beans_dir).unwrap();
        assert_eq!(index.beans.len(), 2);
    }

    #[test]
    fn sync_counts_beans() {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        // Create 5 beans
        for i in 1..=5 {
            let bean = Bean::new(i.to_string(), format!("Task {}", i));
            let slug = title_to_slug(&bean.title);
            bean.to_file(beans_dir.join(format!("{}-{}.md", i, slug)))
                .unwrap();
        }

        let result = cmd_sync(&beans_dir);
        assert!(result.is_ok());

        let index = Index::load(&beans_dir).unwrap();
        assert_eq!(index.beans.len(), 5);
    }

    #[test]
    fn sync_empty_project() {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        let result = cmd_sync(&beans_dir);
        assert!(result.is_ok());

        let index = Index::load(&beans_dir).unwrap();
        assert_eq!(index.beans.len(), 0);
    }

    #[test]
    fn sync_rebuilds_archive_yaml() {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        // Create archive structure with beans
        let archive_dir = beans_dir.join("archive").join("2026").join("03");
        fs::create_dir_all(&archive_dir).unwrap();

        let mut bean1 = Bean::new("10", "Archived ten");
        bean1.status = crate::bean::Status::Closed;
        bean1.is_archived = true;
        let slug1 = title_to_slug(&bean1.title);
        bean1
            .to_file(archive_dir.join(format!("10-{}.md", slug1)))
            .unwrap();

        let mut bean2 = Bean::new("20", "Archived twenty");
        bean2.status = crate::bean::Status::Closed;
        bean2.is_archived = true;
        let slug2 = title_to_slug(&bean2.title);
        bean2
            .to_file(archive_dir.join(format!("20-{}.md", slug2)))
            .unwrap();

        // Sync should rebuild archive.yaml
        cmd_sync(&beans_dir).unwrap();

        assert!(beans_dir.join("archive.yaml").exists());
        let archive = ArchiveIndex::load(&beans_dir).unwrap();
        assert_eq!(archive.beans.len(), 2);
        let ids: Vec<&str> = archive.beans.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"10"));
        assert!(ids.contains(&"20"));
    }

    #[test]
    fn sync_does_not_create_archive_yaml_when_no_archive() {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        cmd_sync(&beans_dir).unwrap();

        // Should NOT create archive.yaml when there's no archive dir
        assert!(!beans_dir.join("archive.yaml").exists());
    }
}
