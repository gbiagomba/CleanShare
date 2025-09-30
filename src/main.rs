mod rules;
mod cleaner;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser};

use crate::cleaner::UrlCleaner;
use crate::rules::RuleSet;

#[derive(Parser, Debug)]
#[command(name = "cleanshare", version, about = "Clean trackers from URLs")]
struct Cli {
    /// Single URL(s) to clean (can be repeated)
    #[arg(short = 'u', long = "url", action = ArgAction::Append)]
    urls: Vec<String>,

    /// Read URLs from file (one per line)
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    /// Optional output file (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Additional rules file (YAML or JSON)
    #[arg(short = 'r', long = "rules")]
    rules: Option<PathBuf>,

    /// Be verbose about non-fatal errors
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,
}

fn read_lines_from_file(path: &PathBuf) -> Result<impl Iterator<Item = String>> {
    let f = File::open(path)
        .with_context(|| format!("Failed to open input file {}", path.display()))?;
    let reader = BufReader::new(f);
    Ok(reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()))
}

fn read_lines_from_stdin() -> Result<impl Iterator<Item = String>> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let buf = BufReader::new(reader);
    Ok(buf
        .lines()
        .filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()))
}

fn write_output(lines: Vec<String>, output: &Option<PathBuf>) -> Result<()> {
    let s = lines.join("\n");
    if let Some(path) = output {
        let mut f = File::create(path)
            .with_context(|| format!("Failed to create output file {}", path.display()))?;
        f.write_all(s.as_bytes())?;
        f.write_all(b"\n")?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(s.as_bytes())?;
        stdout.write_all(b"\n")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load rules: built-in + optional user rules
    let mut rules = RuleSet::builtin();
    if let Some(path) = &cli.rules {
        let user_rules = RuleSet::from_path(path)
            .with_context(|| format!("Failed to load rules file {}", path.display()))?;
        rules.merge(user_rules);
    }

    let cleaner = UrlCleaner::new(rules);

    let mut inputs: Vec<String> = Vec::new();

    // Collect URLs from -u
    inputs.extend(cli.urls.iter().map(|s| s.to_string()));

    // From -f file
    if let Some(path) = &cli.file {
        let iter = read_lines_from_file(path)?;
        inputs.extend(iter);
    }

    // From STDIN if piped
    if !atty::is(atty::Stream::Stdin) {
        let iter = read_lines_from_stdin()?;
        inputs.extend(iter);
    }

    if inputs.is_empty() {
        eprintln!("No input URLs provided. Use -u, -f, or pipe input.");
        std::process::exit(2);
    }

    // Process
    let mut outputs: Vec<String> = Vec::with_capacity(inputs.len());
    for line in inputs {
        match cleaner.clean(&line) {
            Ok(out) => outputs.push(out),
            Err(e) => {
                if cli.verbose {
                    eprintln!("Skipping invalid URL '{}': {}", line, e);
                }
                // Skip invalid lines silently otherwise
            }
        }
    }

    write_output(outputs, &cli.output)?;

    Ok(())
}

