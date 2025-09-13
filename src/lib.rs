//! Core logic for slugpm, extracted for testability.

use std::{io::{self, Write}, path::{Path, PathBuf}};
use anyhow::{Result, Context};
use slug::slugify;

pub fn archive_dir_for_file_pure(parent: &Path) -> PathBuf {
    parent.join("archive")
}

pub fn archive_dir_for_dir_pure(parent: &Path) -> PathBuf {
    parent.parent().unwrap_or(parent).join("archive")
}

pub trait FileOps {
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn open_append(&self, path: &Path) -> Result<Box<dyn Write>>;
}

pub struct RealFileOps;
impl FileOps for RealFileOps {
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to)?;
        Ok(())
    }
    fn open_append(&self, path: &Path) -> Result<Box<dyn Write>> {
        Ok(Box::new(std::fs::OpenOptions::new().create(true).append(true).open(path)?))
    }
}

#[cfg(test)]
pub struct MockFileOps;
#[cfg(test)]
impl FileOps for MockFileOps {
    fn create_dir_all(&self, _path: &Path) -> Result<()> { Ok(()) }
    fn rename(&self, _from: &Path, _to: &Path) -> Result<()> { Ok(()) }
    fn open_append(&self, _path: &Path) -> Result<Box<dyn Write>> {
        struct Sink;
        impl Write for Sink {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        Ok(Box::new(Sink))
    }
}

pub fn archive_move_file_with(file: &Path, ops: &dyn FileOps) -> Result<()> {
    let arch_dir = archive_dir_for_file_pure(file.parent().unwrap());
    ops.create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(file.file_name().unwrap());
    ops.rename(file, &dest)
        .with_context(|| format!("moving {} -> {}", file.display(), dest.display()))?;
    Ok(())
}

pub fn archive_move_dir_with(dir: &Path, ops: &dyn FileOps) -> Result<()> {
    let arch_dir = archive_dir_for_dir_pure(dir.parent().unwrap());
    ops.create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(dir.file_name().unwrap());
    ops.rename(dir, &dest)
        .with_context(|| format!("moving {} -> {}", dir.display(), dest.display()))?;
    Ok(())
}

pub fn archive_append_stdin_with(file: &Path, ops: &dyn FileOps) -> Result<()> {
    let arch_dir = archive_dir_for_file_pure(file.parent().unwrap());
    ops.create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(file.file_name().unwrap());
    let mut f = ops.open_append(&dest)
        .with_context(|| format!("opening {}", dest.display()))?;
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    f.write_all(&buf)?;
    Ok(())
}

pub fn slugify_title(title: &str) -> String {
    slugify(title)
}
