#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::io::{self, Write};

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
    fn test_archive_append_stdin_with_mock() {
        // Simulate stdin using a pipe
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::os::unix::io::{AsRawFd, FromRawFd};
        use std::fs::File;

        // Save the original stdin
        let orig_stdin = io::stdin();
        let (reader, writer) = nix::unistd::pipe().unwrap();
        let mut writer = unsafe { File::from_raw_fd(writer) };
        let reader = unsafe { File::from_raw_fd(reader) };

        // Write to the pipe in a separate thread
        let handle = thread::spawn(move || {
            writer.write_all(b"test input").unwrap();
        });

        // Replace stdin with our pipe
        unsafe {
            libc::dup2(reader.as_raw_fd(), libc::STDIN_FILENO);
        }

        let file = Path::new("/foo/bar.txt");
        let result = archive_append_stdin_with(file, &MockFileOps);
        assert!(result.is_ok());
        handle.join().unwrap();
    }

    #[test]
    fn test_name_command_strips_date() {
        let re = Regex::new(r"^(?P<date>\d{4}-\d{2}-\d{2})(-)?").unwrap();
        let base = "2025-09-13-MyProject";
        let out = re.replace(base, "");
        assert_eq!(out, "MyProject");
    }

    #[test]
    fn test_slugify() {
        let title = "My Project!";
        let slug = slugify(title);
        assert_eq!(slug, "my-project");
    }

    #[test]
    fn test_create_project_dir_pure() {
        let title = "Test Project";
        let slug = slugify(title);
        let dir = Path::new("project").join(&slug);
        assert_eq!(dir, Path::new("project/test-project"));
    }
}
use std::{fs, io::{self, Read, Write}, path::{Path, PathBuf}};
use std::fmt;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use atty::Stream;
use regex::Regex;
use slug::slugify;

#[derive(Parser, Debug)]
#[command(name = "slugpm", version, about = "Project slugs + archiving")]
struct Cli {
    /// Subcommands. If omitted, defaults to `create`.
    #[command(subcommand)]
    command: Option<Cmd>,

    /// Optional title when using the default (create) command; ignored if a subcommand is provided.
    #[arg(global = true)]
    title: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Archive a file or directory.
    ///
    /// - `slugpm archive <path>`:
    ///     * if <path> is a file: moves it to `<parent>/archive/<filename>`
    ///     * if <path> is a dir:  moves it to `<parent>/../archive/<dirname>`
    /// - `slugpm archive <file> -`:
    ///     append STDIN to `<parent>/archive/<filename>` (creating it if needed)
    Archive {
        /// File or directory to archive
        target: PathBuf,
        /// If present and equals "-", append STDIN instead of moving
        #[arg(value_parser = parse_dash, required = false)]
        dash: Option<bool>,
    },

    /// Print the project name excluding a leading YYYY-MM-DD- prefix.
    Name {
        /// Directory whose base name to process
        dirname: PathBuf,
    },
}

// Parse a single literal "-" into true
fn parse_dash(s: &str) -> std::result::Result<bool, String> {
    if s == "-" { Ok(true) } else { Err(format!("expected '-', got {s}")) }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Cmd::from_default_args(cli.title)?) {
        Cmd::Archive { target, dash } => {
            let target = fs::canonicalize(&target)
                .with_context(|| format!("resolving path: {}", target.display()))?;

            if dash.unwrap_or(false) {
                archive_append_stdin(&target)?;
            } else if target.is_file() {
                archive_move_file(&target)?;
            } else if target.is_dir() {
                archive_move_dir(&target)?;
            } else {
                anyhow::bail!("{} is neither file nor directory", target.display());
            }
        }
        Cmd::Name { dirname } => {
            let base = dirname.file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid directory name"))?;
            let re = Regex::new(r"^(?P<date>\d{4}-\d{2}-\d{2})(-)?").unwrap();
            let out = re.replace(base, "");
            println!("{out}");
        }
    }

    Ok(())
}

/// Default command = "create": read title from STDIN's first line if piped, else from args.
/// Creates directory `project/<slug>`.
impl Cmd {
    fn from_default_args(args: Vec<String>) -> Result<Self> {
        if atty::is(Stream::Stdin) {
            // no piped input: use args as a title (joined with spaces)
            let title = if args.is_empty() { anyhow::bail!("missing <title>"); }
                        else { args.join(" ") };
            create_project_dir(&title)?;
        } else {
            // piped: read only first line from stdin
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            let first_line = buf.lines().next().unwrap_or("").trim();
            if first_line.is_empty() { anyhow::bail!("STDIN is empty"); }
            create_project_dir(first_line)?;
        }
        // We already executed; return any placeholder (won't be used)
        Ok(Cmd::Name { dirname: ".".into() })
    }
}

fn create_project_dir(title: &str) -> Result<()> {
    let slug = slugify(title);
    let dir = Path::new("project").join(&slug);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    println!("{}", dir.display());
    Ok(())
}


/// Pure function: Given a file path, returns the archive directory path.
pub fn archive_dir_for_file_pure(parent: &Path) -> PathBuf {
    parent.join("archive")
}

/// Pure function: Given a directory path, returns the archive directory path.
pub fn archive_dir_for_dir_pure(parent: &Path) -> PathBuf {
    parent.parent().unwrap_or(parent).join("archive")
}

// Trait for file operations, so we can mock for tests
pub trait FileOps {
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn open_append(&self, path: &Path) -> Result<Box<dyn Write>>;
}

/// Real file system implementation
pub struct RealFileOps;
impl FileOps for RealFileOps {
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to)?;
        Ok(())
    }
    fn open_append(&self, path: &Path) -> Result<Box<dyn Write>> {
        Ok(Box::new(fs::OpenOptions::new().create(true).append(true).open(path)?))
    }
}

/// Mock file system for tests (in-memory, does nothing)
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


fn archive_move_file(file: &Path) -> Result<()> {
    archive_move_file_with(file, &RealFileOps)
}

fn archive_move_file_with(file: &Path, ops: &dyn FileOps) -> Result<()> {
    let arch_dir = archive_dir_for_file_pure(file.parent().unwrap());
    ops.create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(file.file_name().unwrap());
    ops.rename(file, &dest)
        .with_context(|| format!("moving {} -> {}", file.display(), dest.display()))?;
    println!("{}", dest.display());
    Ok(())
}

fn archive_move_dir(dir: &Path) -> Result<()> {
    archive_move_dir_with(dir, &RealFileOps)
}

fn archive_move_dir_with(dir: &Path, ops: &dyn FileOps) -> Result<()> {
    let arch_dir = archive_dir_for_dir_pure(dir.parent().unwrap());
    ops.create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(dir.file_name().unwrap());
    ops.rename(dir, &dest)
        .with_context(|| format!("moving {} -> {}", dir.display(), dest.display()))?;
    println!("{}", dest.display());
    Ok(())
}

fn archive_append_stdin(file: &Path) -> Result<()> {
    archive_append_stdin_with(file, &RealFileOps)
}

fn archive_append_stdin_with(file: &Path, ops: &dyn FileOps) -> Result<()> {
    let arch_dir = archive_dir_for_file_pure(file.parent().unwrap());
    ops.create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(file.file_name().unwrap());
    let mut f = ops.open_append(&dest)
        .with_context(|| format!("opening {}", dest.display()))?;
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    f.write_all(&buf)?;
    println!("{}", dest.display());
    Ok(())
}
