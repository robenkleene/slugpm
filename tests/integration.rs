use slugpm::*;
use std::path::Path;

#[test]
fn test_archive_dir_for_file_pure() {
    let parent = Path::new("/foo/bar");
    let arch = archive_dir_for_file_pure(parent);
    assert_eq!(arch, Path::new("/foo/bar/archive"));
}

#[test]
fn test_archive_dir_for_dir_pure() {
    let parent = Path::new("/foo/bar");
    let arch = archive_dir_for_dir_pure(parent);
    assert_eq!(arch, Path::new("/foo/archive"));
}

#[test]
fn test_archive_move_file_with_mock() {
    let file = Path::new("/foo/bar.txt");
    let result = archive_move_file_with(file, &MockFileOps);
    assert!(result.is_ok());
}

#[test]
fn test_archive_move_dir_with_mock() {
    let dir = Path::new("/foo/bar");
    let result = archive_move_dir_with(dir, &MockFileOps);
    assert!(result.is_ok());
}

#[test]
fn test_slugify_title() {
    let title = "My Project!";
    let slug = slugify_title(title);
    assert_eq!(slug, "my-project");
}
