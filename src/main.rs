use std::{fs, io::{self, Read, Write}, path::{Path, PathBuf}};
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

fn archive_dir_for_file(file: &Path) -> PathBuf {
    file.parent().unwrap().join("archive")
}
fn archive_dir_for_dir(dir: &Path) -> PathBuf {
    // “…../<parent>/../archive/<dirname> relative to <dirname>”
    let parent = dir.parent().unwrap();
    parent.parent().unwrap_or(parent).join("archive")
}

fn archive_move_file(file: &Path) -> Result<()> {
    let arch_dir = archive_dir_for_file(file);
    fs::create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(file.file_name().unwrap());
    fs::rename(file, &dest)
        .with_context(|| format!("moving {} -> {}", file.display(), dest.display()))?;
    println!("{}", dest.display());
    Ok(())
}

fn archive_move_dir(dir: &Path) -> Result<()> {
    let arch_dir = archive_dir_for_dir(dir);
    fs::create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(dir.file_name().unwrap());
    fs::rename(dir, &dest)
        .with_context(|| format!("moving {} -> {}", dir.display(), dest.display()))?;
    println!("{}", dest.display());
    Ok(())
}

fn archive_append_stdin(file: &Path) -> Result<()> {
    let arch_dir = archive_dir_for_file(file);
    fs::create_dir_all(&arch_dir)?;
    let dest = arch_dir.join(file.file_name().unwrap());
    let mut f = fs::OpenOptions::new()
        .create(true).append(true).open(&dest)
        .with_context(|| format!("opening {}", dest.display()))?;
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    f.write_all(&buf)?;
    println!("{}", dest.display());
    Ok(())
}
