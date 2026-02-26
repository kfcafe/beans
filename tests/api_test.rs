use std::fs;
use tempfile::TempDir;

use bn::api::*;

/// Set up a temporary .beans/ directory with a sample bean.
fn setup_test_env() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let beans_dir = dir.path().join(".beans");
    fs::create_dir_all(&beans_dir).unwrap();

    // Write minimal config
    fs::write(beans_dir.join("config.yaml"), "next_id: 2\n").unwrap();

    // Write a sample bean
    let bean = Bean::new("1", "Sample task");
    let slug = bn::util::title_to_slug(&bean.title);
    bean.to_file(beans_dir.join(format!("1-{}.md", slug)))
        .unwrap();

    (dir, beans_dir)
}

#[test]
fn api_re_exports_core_types() {
    // This test verifies that core types are accessible via bn::api
    let _status = Status::Open;
    let _result = RunResult::Pass;
    let bean = Bean::new("1", "Test");
    assert_eq!(bean.id, "1");
    assert_eq!(bean.status, Status::Open);
}

#[test]
fn api_get_bean_loads_by_id() {
    let (_dir, beans_dir) = setup_test_env();
    let bean = get_bean(&beans_dir, "1").unwrap();
    assert_eq!(bean.id, "1");
    assert_eq!(bean.title, "Sample task");
    assert_eq!(bean.status, Status::Open);
}

#[test]
fn api_get_bean_not_found() {
    let (_dir, beans_dir) = setup_test_env();
    let result = get_bean(&beans_dir, "999");
    assert!(result.is_err());
}

#[test]
fn api_load_index_returns_entries() {
    let (_dir, beans_dir) = setup_test_env();
    let index = load_index(&beans_dir).unwrap();
    assert_eq!(index.beans.len(), 1);
    assert_eq!(index.beans[0].id, "1");
    assert_eq!(index.beans[0].title, "Sample task");
}

#[test]
fn api_find_beans_dir_discovers_directory() {
    let (dir, _beans_dir) = setup_test_env();
    let found = find_beans_dir(dir.path()).unwrap();
    assert!(found.ends_with(".beans"));
    assert!(found.is_dir());
}

#[test]
fn api_types_are_serializable() {
    let bean = Bean::new("1", "Serializable");
    let json = serde_json::to_string(&bean).unwrap();
    assert!(json.contains("Serializable"));

    let entry = IndexEntry::from(&bean);
    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("Serializable"));
}

#[test]
fn api_graph_functions_accessible() {
    let (_dir, beans_dir) = setup_test_env();
    let index = load_index(&beans_dir).unwrap();

    // No cycles in a single-bean graph
    let cycles = find_all_cycles(&index).unwrap();
    assert!(cycles.is_empty());

    // Full graph renders
    let graph = build_full_graph(&index).unwrap();
    assert!(graph.contains("Sample task"));
}
