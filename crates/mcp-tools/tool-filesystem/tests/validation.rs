use std::path::{Path, PathBuf};
use tool_filesystem::validation::{lexical_normalize, validate_path_with_root};

#[test]
fn lexical_normalize_simple() {
    assert_eq!(lexical_normalize(Path::new("/a/b/c")), PathBuf::from("/a/b/c"));
}

#[test]
fn lexical_normalize_dot() {
    assert_eq!(lexical_normalize(Path::new("/a/./b")), PathBuf::from("/a/b"));
}

#[test]
fn lexical_normalize_dotdot() {
    assert_eq!(lexical_normalize(Path::new("/a/b/../c")), PathBuf::from("/a/c"));
}

#[test]
fn lexical_normalize_dotdot_beyond_root() {
    assert_eq!(lexical_normalize(Path::new("/a/../../etc/passwd")), PathBuf::from("/etc/passwd"));
}

#[test]
fn lexical_normalize_trailing_slash() {
    assert_eq!(lexical_normalize(Path::new("/a/b/")), PathBuf::from("/a/b"));
}

#[test]
fn lexical_normalize_relative() {
    assert_eq!(lexical_normalize(Path::new("sub/../file.txt")), PathBuf::from("file.txt"));
}

#[test]
fn lexical_normalize_empty_components() {
    assert_eq!(lexical_normalize(Path::new("/a//b")), PathBuf::from("/a/b"));
}

#[test]
fn validate_allows_subpath() {
    assert!(validate_path_with_root("/project/src/main.rs", Path::new("/project")).is_ok());
}

#[test]
fn validate_rejects_traversal() {
    assert!(validate_path_with_root("/project/../../etc/passwd", Path::new("/project")).is_err());
}

#[test]
fn validate_rejects_absolute_escape() {
    assert!(validate_path_with_root("/etc/passwd", Path::new("/project")).is_err());
}

#[test]
fn validate_allows_relative_within_root() {
    assert!(validate_path_with_root("src/main.rs", Path::new("/project")).is_ok());
}

#[test]
fn validate_rejects_relative_traversal() {
    assert!(validate_path_with_root("../../etc/passwd", Path::new("/project")).is_err());
}

#[test]
fn validate_allows_root_itself() {
    assert!(validate_path_with_root("/project", Path::new("/project")).is_ok());
}

#[test]
fn validate_allows_dot() {
    assert!(validate_path_with_root(".", Path::new("/project")).is_ok());
}

#[test]
fn validate_resolves_dotdot_in_middle() {
    let result = validate_path_with_root("sub/../file.txt", Path::new("/project"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/project/file.txt"));
}