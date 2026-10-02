//! Creates new projects from this starter template. Run it from a clone of the template:
//! `cargo starter new <dir>` (an alias in `.cargo/config.toml`).
//!
//! How a project is generated (see also "Maintaining the template" in CLAUDE.md):
//! 1. Frontends other than `vanilla` replace the vanilla files with the overlay in
//!    `starter/frontends/<name>/`.
//! 2. Without the example, the notes files are deleted.
//! 3. Marker comments are resolved in every text file. A line ending in
//!    `// starter:<feature>` is kept, with the marker removed, only when the feature is on.
//!    Lines between `// starter:<feature>:begin` and `// starter:<feature>:end` are kept
//!    only when it is on. `!<feature>` negates. Any comment style works:
//!    `//`, `#`, `/* */`, `<!-- -->` and `{/* */}`.
//!    The features are `web`, `example`, the chosen frontend, and `template`, which is
//!    always off and marks content that only makes sense in the template itself.
//! 4. `kudamerah` is renamed to the project name in every case style.
//! 5. The template-only paths (`starter/`, `.cargo/`) are deleted, then
//!    `cargo update -w`, `cargo fmt` and git run.

use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

/// The template clone this binary was built from, which is what `new` copies.
const TEMPLATE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

/// Paths that only make sense in the template. They are deleted from generated projects.
const TEMPLATE_PATHS: &[&str] = &["starter", ".cargo"];

/// Paths owned by the default (vanilla) frontend. They are replaced when another frontend is chosen.
const FRONTEND_PATHS: &[&str] = &[
    "apps/web",
    "apps/server/src/web.rs",
    "apps/server/tests/web.rs",
];

/// Paths owned by the notes example. They are deleted when the example is not wanted.
const EXAMPLE_PATHS: &[&str] = &[
    "apps/server/src/modules/notes.rs",
    "apps/server/tests/notes.rs",
    "apps/server/migrations/20261002000000_create_notes.sql",
    "apps/web/notes.js",
    "apps/web/src/Notes.tsx",
];

/// Directories that are never copied or rewritten.
const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", "data", "dist"];

const USAGE: &str = "\
Create a new project from the kudamerah starter template.

Usage (from the template clone):
  cargo starter new <dir> [options]

Options:
  --name <name>            project name in kebab-case (default: directory name)
  --description <text>     one-line description for the README
  --frontend <kind>        vanilla | htmx | solid | none (default: vanilla)
  --example, --no-example  include the notes CRUD example (default: yes)
  --git, --no-git          commit the result to git (default: yes)
  -y, --yes                accept defaults and ask nothing
  -h, --help               show this help
";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        print!("{USAGE}");
        return Ok(());
    };
    if ["-h", "--help", "help"].contains(&command.as_str())
        || rest.iter().any(|a| a == "-h" || a == "--help")
    {
        print!("{USAGE}");
        return Ok(());
    }
    if command != "new" {
        return Err(format!("unknown command `{command}` (see --help)").into());
    }
    let opts = parse_args(rest)?;
    let dir = opts
        .dir
        .clone()
        .ok_or("usage: cargo starter new <dir> [options]")?;
    new_project(Path::new(&dir), &opts)
}

fn new_project(dest: &Path, opts: &Options) -> Result<()> {
    if dest.exists() && fs::read_dir(dest)?.next().is_some() {
        return Err(format!("`{}` already exists and is not empty", dest.display()).into());
    }
    let plan = Plan::gather(opts, &dir_name(dest))?;
    println!("\nCreating {} in {}", plan.name.kebab, dest.display());
    // `dest` was empty or missing, so a failed run can be cleaned up entirely.
    copy_template(Path::new(TEMPLATE_ROOT), dest)
        .and_then(|()| configure(dest, &plan))
        .inspect_err(|_| drop(fs::remove_dir_all(dest)))?;
    print_next_steps(dest, &plan);
    Ok(())
}

// ---------------------------------------------------------------------------
// Options and the wizard

#[derive(Default)]
struct Options {
    dir: Option<String>,
    name: Option<String>,
    description: Option<String>,
    frontend: Option<Frontend>,
    example: Option<bool>,
    git: Option<bool>,
    yes: bool,
}

fn parse_args(args: &[String]) -> Result<Options> {
    let mut opts = Options::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
            _ => (arg.as_str(), None),
        };
        let mut value = || -> Result<String, String> {
            inline
                .or_else(|| it.next().map(String::as_str))
                .map(str::to_owned)
                .ok_or_else(|| format!("`{flag}` needs a value"))
        };
        match flag {
            "--name" => opts.name = Some(value()?),
            "--description" => opts.description = Some(value()?),
            "--frontend" => {
                let v = value()?;
                let frontend = Frontend::parse(&v).ok_or_else(|| {
                    format!("unknown frontend `{v}` (expected vanilla, htmx, solid or none)")
                })?;
                opts.frontend = Some(frontend);
            }
            "--example" => opts.example = Some(true),
            "--no-example" => opts.example = Some(false),
            "--git" => opts.git = Some(true),
            "--no-git" => opts.git = Some(false),
            "-y" | "--yes" => opts.yes = true,
            f if f.starts_with('-') => return Err(format!("unknown option `{f}`").into()),
            _ if opts.dir.is_none() => opts.dir = Some(arg.clone()),
            _ => return Err(format!("unexpected argument `{arg}`").into()),
        }
    }
    Ok(opts)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Frontend {
    Vanilla,
    Htmx,
    Solid,
    None,
}

impl Frontend {
    const ALL: [Self; 4] = [Self::Vanilla, Self::Htmx, Self::Solid, Self::None];

    fn name(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Htmx => "htmx",
            Self::Solid => "solid",
            Self::None => "none",
        }
    }

    fn about(self) -> &'static str {
        match self {
            Self::Vanilla => "static HTML/CSS/JS, no build step",
            Self::Htmx => "server-rendered HTML (maud) + htmx",
            Self::Solid => "SolidJS + Vite SPA (needs Node)",
            Self::None => "JSON API only",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.name() == s)
    }
}

/// Every form of the project name that replaces a form of `kudamerah`.
#[derive(Debug, PartialEq)]
struct Name {
    kebab: String,
    snake: String,
    title: String,
    screaming: String,
}

impl Name {
    fn parse(s: &str) -> Result<Self, String> {
        let valid = s.starts_with(|c: char| c.is_ascii_lowercase())
            && s.split('-').all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            });
        if !valid {
            return Err(format!(
                "`{s}` is not a valid name: use lowercase letters, digits and single hyphens, \
                 starting with a letter (e.g. my-app)"
            ));
        }
        let snake = s.replace('-', "_");
        Ok(Self {
            kebab: s.to_owned(),
            title: s.split('-').map(capitalize).collect::<Vec<_>>().join(" "),
            screaming: snake.to_ascii_uppercase(),
            snake,
        })
    }
}

struct Plan {
    name: Name,
    description: String,
    frontend: Frontend,
    example: bool,
    git: bool,
}

impl Plan {
    /// Takes answers from flags, asks for the rest (or uses defaults with `--yes`).
    fn gather(opts: &Options, dir_name: &str) -> Result<Self> {
        let ask = !opts.yes;
        if ask {
            println!("New project wizard (Enter accepts the [default])\n");
        }
        let default_name = Some(slugify(dir_name))
            .filter(|s| Name::parse(s).is_ok())
            .unwrap_or_else(|| "my-app".to_owned());

        let name = match &opts.name {
            Some(name) => Name::parse(name)?,
            None if !ask => Name::parse(&default_name)?,
            None => loop {
                match Name::parse(&prompt("Project name", &default_name)?) {
                    Ok(name) => break name,
                    Err(err) => println!("  {err}"),
                }
            },
        };
        let description = match &opts.description {
            Some(d) => d.clone(),
            None if ask => prompt("Description (optional)", "")?,
            None => String::new(),
        };
        let frontend = match opts.frontend {
            Some(f) => f,
            None if ask => choose_frontend()?,
            None => Frontend::Vanilla,
        };
        let example = match opts.example {
            Some(e) => e,
            None if ask => confirm("Include the notes example (CRUD, migration, tests)?", true)?,
            None => true,
        };
        let git = match opts.git {
            Some(g) => g,
            None if ask => confirm("Commit the result to git?", true)?,
            None => true,
        };
        Ok(Self {
            name,
            description,
            frontend,
            example,
            git,
        })
    }

    /// Whether a marker feature is on. `None` for unknown features.
    fn feature(&self, feature: &str) -> Option<bool> {
        Some(match feature {
            "web" => self.frontend != Frontend::None,
            "example" => self.example,
            "template" => false,
            other => self.frontend == Frontend::parse(other)?,
        })
    }
}

fn prompt(label: &str, default: &str) -> io::Result<String> {
    if default.is_empty() {
        print!("{label}: ");
    } else {
        print!("{label} [{default}]: ");
    }
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    let line = line.trim();
    Ok(if line.is_empty() { default } else { line }.to_owned())
}

fn confirm(label: &str, default: bool) -> io::Result<bool> {
    let hint = if default { "[Y/n]" } else { "[y/N]" };
    loop {
        match prompt(&format!("{label} {hint}"), "")?
            .to_ascii_lowercase()
            .as_str()
        {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("  answer y or n"),
        }
    }
}

fn choose_frontend() -> io::Result<Frontend> {
    println!("Frontend:");
    for (i, f) in Frontend::ALL.iter().enumerate() {
        println!("  {}) {:<8} {}", i + 1, f.name(), f.about());
    }
    loop {
        let answer = prompt("Choose", "1")?;
        let picked = answer
            .parse::<usize>()
            .ok()
            .and_then(|n| Frontend::ALL.get(n.checked_sub(1)?).copied())
            .or_else(|| Frontend::parse(&answer));
        match picked {
            Some(f) => return Ok(f),
            None => println!("  enter 1-4 or a name"),
        }
    }
}

// ---------------------------------------------------------------------------
// Generation

fn copy_template(src: &Path, dest: &Path) -> Result<()> {
    for rel in template_files(src)? {
        let from = src.join(&rel);
        if from.is_file() {
            copy_file(&from, &dest.join(&rel))?;
        }
    }
    Ok(())
}

/// The files git would commit (tracked and untracked, minus ignored), or every file outside git.
fn template_files(src: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(src)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .stderr(Stdio::null())
        .output();
    match output {
        Ok(out) if out.status.success() => Ok(out
            .stdout
            .split(|&b| b == 0)
            .filter(|p| !p.is_empty())
            .map(|p| PathBuf::from(String::from_utf8_lossy(p).into_owned()))
            .collect()),
        _ => {
            let mut files = Vec::new();
            walk(src, src, &mut files)?;
            Ok(files)
        }
    }
}

fn configure(root: &Path, plan: &Plan) -> Result<()> {
    let starter = root.join("starter");

    if plan.frontend != Frontend::Vanilla {
        for path in FRONTEND_PATHS {
            remove(&root.join(path))?;
        }
        let overlay = starter.join("frontends").join(plan.frontend.name());
        if overlay.is_dir() {
            let mut files = Vec::new();
            walk(&overlay, &overlay, &mut files)?;
            for rel in files {
                copy_file(&overlay.join(&rel), &root.join(&rel))?;
            }
        }
    }
    if !plan.example {
        for path in EXAMPLE_PATHS {
            remove(&root.join(path))?;
        }
    }
    fs::rename(starter.join("README.project.md"), root.join("README.md"))?;
    for path in TEMPLATE_PATHS {
        remove(&root.join(path))?;
    }

    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    for rel in files {
        if rel == Path::new("Cargo.lock") {
            continue; // `cargo update` below rewrites it
        }
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            continue; // not UTF-8 text
        };
        let rendered = render(&text, plan).map_err(|err| format!("{}: {err}", rel.display()))?;
        if rendered != text {
            fs::write(&path, rendered)?;
        }
    }

    let cargo = |args: &[&str]| {
        let mut cmd = Command::new("cargo");
        cmd.args(args).current_dir(root);
        step(&format!("cargo {}", args.join(" ")), &mut cmd)
    };
    cargo(&["update", "--workspace", "--quiet"]);
    cargo(&["fmt", "--all"]);

    if plan.git {
        let git = |args: &[&str]| {
            let mut cmd = Command::new("git");
            cmd.args(args).current_dir(root).stdout(Stdio::null());
            step(&format!("git {}", args.join(" ")), &mut cmd)
        };
        let _ = git(&["init", "--quiet", "--initial-branch=main"])
            && git(&["add", "--all"])
            && git(&[
                "commit",
                "--quiet",
                "-m",
                "Initial commit from kudamerah starter",
            ]);
    }
    Ok(())
}

/// Applies markers, the description and the project name to one file.
fn render(text: &str, plan: &Plan) -> Result<String, String> {
    let text = apply_markers(text, &|feature| plan.feature(feature))?;
    let text = if plan.description.is_empty() {
        text.replace("{{description}}\n\n", "")
            .replace("{{description}}", "")
    } else {
        text.replace("{{description}}", &plan.description)
    };
    Ok(rename(&text, &plan.name))
}

fn rename(text: &str, name: &Name) -> String {
    text.replace("kudamerah_", &format!("{}_", name.snake))
        .replace("KUDAMERAH", &name.screaming)
        .replace("Kudamerah", &name.title)
        .replace("kudamerah", &name.kebab)
}

#[derive(Debug, PartialEq)]
enum Marker<'a> {
    Begin(&'a str),
    End(&'a str),
    Line { tag: &'a str, code: &'a str },
}

const OPENERS: [&str; 5] = ["{/*", "<!--", "/*", "//", "#"];
const CLOSERS: [&str; 3] = ["*/}", "-->", "*/"];

/// Recognises a marker comment at the end of a line.
fn parse_marker(line: &str) -> Option<Marker<'_>> {
    let at = line.rfind("starter:")?;
    let rest = &line[at + "starter:".len()..];
    let (tag, tail) = rest.split_at(rest.find(char::is_whitespace).unwrap_or(rest.len()));
    let tail = tail.trim();
    let tag_ok = !tag.is_empty()
        && tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '!' || c == ':');
    if !tag_ok || !(tail.is_empty() || CLOSERS.contains(&tail)) {
        return None;
    }
    let before = line[..at].trim_end();
    let opener = OPENERS.iter().find(|o| before.ends_with(**o))?;
    let code = before[..before.len() - opener.len()].trim_end();
    if let Some(tag) = tag.strip_suffix(":begin") {
        return code.is_empty().then_some(Marker::Begin(tag));
    }
    if let Some(tag) = tag.strip_suffix(":end") {
        return code.is_empty().then_some(Marker::End(tag));
    }
    Some(Marker::Line { tag, code })
}

fn apply_markers(text: &str, is_on: &dyn Fn(&str) -> Option<bool>) -> Result<String, String> {
    let eval = |tag: &str, line: usize| -> Result<bool, String> {
        let (negate, feature) = match tag.strip_prefix('!') {
            Some(feature) => (true, feature),
            None => (false, tag),
        };
        is_on(feature)
            .map(|on| on != negate)
            .ok_or_else(|| format!("line {line}: unknown feature `{feature}`"))
    };

    let mut out = String::with_capacity(text.len());
    let mut blocks: Vec<(&str, bool)> = Vec::new();
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let n = i + 1;
        let content = line.trim_end_matches(['\n', '\r']);
        let eol = &line[content.len()..];
        let visible = blocks.iter().all(|&(_, on)| on);
        match parse_marker(content) {
            Some(Marker::Begin(tag)) => blocks.push((tag, eval(tag, n)?)),
            Some(Marker::End(tag)) => match blocks.pop() {
                Some((open, _)) if open == tag => {}
                _ => {
                    return Err(format!(
                        "line {n}: `starter:{tag}:end` has no matching begin"
                    ));
                }
            },
            Some(Marker::Line { tag, code }) => {
                if visible && eval(tag, n)? && !code.is_empty() {
                    out.push_str(code);
                    out.push_str(eol);
                }
            }
            None if visible => out.push_str(line),
            None => {}
        }
    }
    match blocks.pop() {
        Some((tag, _)) => Err(format!("`starter:{tag}:begin` is never closed")),
        None => Ok(out),
    }
}

fn print_next_steps(dir: &Path, plan: &Plan) {
    let example = if plan.example { " + notes example" } else { "" };
    println!(
        "\nDone: {} ({} frontend{example}).\n\nNext steps:",
        plan.name.kebab,
        plan.frontend.name()
    );
    println!("  cd {}", dir.display());
    if plan.frontend == Frontend::Solid {
        println!("  cargo run                                     # API on http://127.0.0.1:3000");
        println!("  (cd apps/web && npm install && npm run dev)   # UI on http://localhost:5173");
    } else {
        println!("  cargo run      # http://127.0.0.1:3000");
    }
    println!("\nREADME.md has the details. CLAUDE.md has the conventions.");
}

// ---------------------------------------------------------------------------
// Small helpers

fn step(label: &str, cmd: &mut Command) -> bool {
    match cmd.status() {
        Ok(status) if status.success() => true,
        _ => {
            eprintln!("  warning: `{label}` failed; run it yourself");
            false
        }
    }
}

/// Collects files under `dir` as paths relative to `root`, skipping `SKIP_DIRS`.
fn walk(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            if !SKIP_DIRS
                .iter()
                .any(|s| path.file_name() == Some(OsStr::new(s)))
            {
                walk(root, &path, files)?;
            }
        } else if let Ok(rel) = path.strip_prefix(root) {
            files.push(rel.to_path_buf());
        }
    }
    Ok(())
}

fn copy_file(from: &Path, to: &Path) -> io::Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(from, to).map(drop)
}

fn remove(path: &Path) -> io::Result<()> {
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    match result {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .or_else(|| {
            let abs = path.canonicalize().ok()?;
            Some(abs.file_name()?.to_string_lossy().into_owned())
        })
        .unwrap_or_default()
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_owned()
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn features(f: &str) -> Option<bool> {
        match f {
            "on" => Some(true),
            "off" => Some(false),
            _ => None,
        }
    }

    #[test]
    fn parses_markers_in_any_comment_style() {
        assert_eq!(
            parse_marker("mod web; // starter:web"),
            Some(Marker::Line {
                tag: "web",
                code: "mod web;"
            })
        );
        assert_eq!(
            parse_marker(r#"maud = "1" # starter:htmx"#),
            Some(Marker::Line {
                tag: "htmx",
                code: r#"maud = "1""#
            })
        );
        assert_eq!(
            parse_marker("      <Notes /> {/* starter:example */}"),
            Some(Marker::Line {
                tag: "example",
                code: "      <Notes />"
            })
        );
        assert_eq!(
            parse_marker("    <!-- starter:example:begin -->"),
            Some(Marker::Begin("example"))
        );
        assert_eq!(
            parse_marker("  // starter:!web:end"),
            Some(Marker::End("!web"))
        );
        assert_eq!(
            parse_marker("/* starter:solid */"),
            Some(Marker::Line {
                tag: "solid",
                code: ""
            })
        );
    }

    #[test]
    fn ignores_text_that_only_mentions_markers() {
        assert_eq!(
            parse_marker("Wrap it in `// starter:x:begin` and `// starter:x:end`."),
            None
        );
        assert_eq!(parse_marker("let s = \"starter:web\";"), None);
        assert_eq!(parse_marker("code(); // starter:web extra"), None);
        assert_eq!(parse_marker("code(); // starter:web:begin"), None); // blocks need their own line
    }

    #[test]
    fn applies_line_and_block_markers() {
        let text = "a\nb // starter:on\nc // starter:off\nd // starter:!off\n\
                    // starter:off:begin\ne\n// starter:on:begin\nf\n// starter:on:end\n// starter:off:end\n\
                    # starter:on:begin\ng\n# starter:on:end\nh\n";
        assert_eq!(apply_markers(text, &features).unwrap(), "a\nb\nd\ng\nh\n");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(
            apply_markers("a // starter:on\r\nb\r\n", &features).unwrap(),
            "a\r\nb\r\n"
        );
    }

    #[test]
    fn rejects_unknown_features_and_unbalanced_blocks() {
        assert!(
            apply_markers("x // starter:typo\n", &features)
                .unwrap_err()
                .contains("typo")
        );
        assert!(apply_markers("// starter:on:begin\n", &features).is_err());
        assert!(apply_markers("// starter:on:end\n", &features).is_err());
        assert!(apply_markers("// starter:on:begin\n// starter:off:end\n", &features).is_err());
    }

    #[test]
    fn derives_name_forms() {
        let name = Name::parse("my-app2").unwrap();
        assert_eq!(name.snake, "my_app2");
        assert_eq!(name.title, "My App2");
        assert_eq!(name.screaming, "MY_APP2");
        for bad in ["", "My-App", "1app", "my--app", "my-app-", "my_app"] {
            assert!(Name::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn renames_every_case_style() {
        let name = Name::parse("red-fox").unwrap();
        let text = "kudamerah-server kudamerah_server Kudamerah KUDAMERAH_PORT data/kudamerah.db";
        assert_eq!(
            rename(text, &name),
            "red-fox-server red_fox_server Red Fox RED_FOX_PORT data/red-fox.db"
        );
    }

    #[test]
    fn slugifies_directory_names() {
        assert_eq!(slugify("My Cool_App!"), "my-cool-app");
        assert_eq!(slugify("app"), "app");
    }
}
