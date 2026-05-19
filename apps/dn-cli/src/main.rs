use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dn_runtime::{
    available_profile_entries, available_profiles, effective_profile, scan_repository, Diagnostic,
    OutputFormat, ScanOptions,
};

#[derive(Parser, Debug)]
#[command(name = "dn-cli")]
#[command(version)]
#[command(about = "Terminal code review and repository audit assistant")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan a repository path and emit a repository review.
    Scan(ScanCommand),
    /// Review is a first-class alias for scan.
    Review(ScanCommand),
    /// Inspect built-in and local profiles.
    Profiles(ProfileCommands),
    /// Validate a profile file.
    ValidateProfile(ValidateProfileCommand),
    /// Run lightweight environment and integration checks.
    Doctor(DoctorCommand),
}

#[derive(Args, Debug, Clone)]
struct ScanCommand {
    /// Path to scan (repository root or folder)
    path: String,
    #[arg(long, default_value = "quick")]
    profile: String,
    #[arg(long, conflicts_with = "markdown")]
    json: bool,
    #[arg(long, alias = "md", conflicts_with = "json")]
    markdown: bool,
    #[arg(long)]
    content: bool,
    #[arg(long)]
    hidden: bool,
    #[arg(long)]
    python_worker: bool,
    #[arg(long, value_enum, default_value = "none")]
    fail_on: FailOnSeverity,
    #[arg(long)]
    summary_only: bool,
    #[arg(long)]
    strict_integrations: bool,
    #[arg(long)]
    #[arg(value_parser = parse_positive_usize)]
    max_files: Option<usize>,
}

#[derive(Subcommand, Debug)]
enum ProfileSubcommands {
    List {
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        root: String,
    },
    Show {
        name_or_path: String,
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        root: String,
    },
}

#[derive(Args, Debug)]
struct ProfileCommands {
    #[command(subcommand)]
    command: ProfileSubcommands,
}

#[derive(Args, Debug)]
struct ValidateProfileCommand {
    path: String,
    #[arg(long)]
    json: bool,
    #[arg(default_value = ".")]
    root: String,
}

#[derive(Args, Debug)]
struct DoctorCommand {
    #[arg(long)]
    json: bool,
    #[arg(default_value = ".")]
    root: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum FailOnSeverity {
    None,
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl FailOnSeverity {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("invalid positive integer: {value}"))?;
    if parsed == 0 {
        return Err("value must be greater than zero".to_string());
    }
    Ok(parsed)
}

fn render_markdown(report: &dn_runtime::ScanReport) -> String {
    let mut output = String::new();
    output.push_str("# dn-kernel Review Report\n\n");
    output.push_str("## Execution Summary\n\n");
    output.push_str(&format!("- Root: `{}`\n", report.metadata.root));
    output.push_str(&format!("- Profile: `{}`\n", report.metadata.profile));
    output.push_str(&format!(
        "- Profile source: `{}`\n",
        report.metadata.profile_source
    ));
    output.push_str(&format!("- Command: `{}`\n", report.metadata.command));
    output.push_str(&format!("- Duration: {}ms\n", report.metadata.duration_ms));
    output.push_str(&format!("- Truncated: {}\n", report.metadata.truncated));
    output.push_str(&format!(
        "- Summary only: {}\n\n",
        report.metadata.summary_only
    ));

    output.push_str("## Integration Status\n\n");
    output.push_str(&format!(
        "- Worker: enabled={} mode=`{}` used={} strict={}\n",
        report.integrations.worker.enabled,
        report.integrations.worker.mode,
        report.integrations.worker.used,
        report.integrations.worker.strict
    ));
    output.push_str(&format!(
        "- Provider: enabled={} mode=`{}` used={} strict={}\n\n",
        report.integrations.provider.enabled,
        report.integrations.provider.mode,
        report.integrations.provider.used,
        report.integrations.provider.strict
    ));

    output.push_str("## Severity Breakdown\n\n");
    output.push_str(&format!(
        "- Findings total: {}\n",
        report.stats.findings_total
    ));
    output.push_str(&format!(
        "- info={} low={} medium={} high={} critical={}\n\n",
        report.stats.severity_breakdown.info,
        report.stats.severity_breakdown.low,
        report.stats.severity_breakdown.medium,
        report.stats.severity_breakdown.high,
        report.stats.severity_breakdown.critical
    ));

    output.push_str("## Findings\n\n");
    let mut found_any = false;
    for file in &report.files {
        if file.findings.is_empty() {
            continue;
        }
        found_any = true;
        output.push_str(&format!("### `{}`\n", file.path));
        for finding in &file.findings {
            let line_suffix = finding
                .line
                .map(|line| format!(" line {}", line))
                .unwrap_or_default();
            output.push_str(&format!(
                "- **{}** [{} / {}{}] {}\n",
                finding.severity, finding.origin, finding.rule, line_suffix, finding.message
            ));
        }
        output.push('\n');
    }
    if !found_any {
        output.push_str("*No findings were reported for the current profile.*\n\n");
    }

    output.push_str("## Diagnostics\n\n");
    if report.diagnostics.is_empty() {
        output.push_str("- No diagnostics reported.\n");
    } else {
        for diagnostic in &report.diagnostics {
            output.push_str(&format!(
                "- {} [{}:{}] {}\n",
                diagnostic.level, diagnostic.source, diagnostic.code, diagnostic.message
            ));
        }
    }
    output
}

fn print_report(report: &dn_runtime::ScanReport, want_json: bool, want_markdown: bool) {
    if want_json {
        println!(
            "{}",
            serde_json::to_string_pretty(report).unwrap_or_else(|err| {
                eprintln!("error: failed to serialize report: {err}");
                process::exit(1);
            })
        );
        return;
    }
    if want_markdown {
        print!("{}", render_markdown(report));
        return;
    }

    println!("schema_version={}", report.schema_version);
    println!("root={}", report.metadata.root);
    println!("profile={}", report.metadata.profile);
    println!("profile_source={}", report.metadata.profile_source);
    println!("command={}", report.metadata.command);
    println!("worker={}", report.integrations.worker.mode);
    println!("provider={}", report.integrations.provider.mode);
    println!("files_discovered={}", report.stats.files_discovered);
    println!("files_scanned={}", report.stats.files_scanned);
    println!("files_selected={}", report.stats.files_selected);
    println!("findings={}", report.stats.findings_total);
    println!("summary={}", report.summary);
    if !report.diagnostics.is_empty() {
        println!("diagnostics:");
        for diagnostic in &report.diagnostics {
            println!(
                "  - {} [{}:{}] {}",
                diagnostic.level, diagnostic.source, diagnostic.code, diagnostic.message
            );
        }
    }
}

fn severity_triggered(report: &dn_runtime::ScanReport, threshold: FailOnSeverity) -> bool {
    if threshold == FailOnSeverity::None {
        return false;
    }
    let rank = match threshold {
        FailOnSeverity::None => 0,
        FailOnSeverity::Info => 1,
        FailOnSeverity::Low => 2,
        FailOnSeverity::Medium => 3,
        FailOnSeverity::High => 4,
        FailOnSeverity::Critical => 5,
    };
    report
        .files
        .iter()
        .flat_map(|file| file.findings.iter())
        .any(|finding| match finding.severity.as_str() {
            "critical" => 5,
            "high" => 4,
            "medium" => 3,
            "low" => 2,
            _ => 1,
        } >= rank)
}

fn run_scan(command_name: &str, command: ScanCommand) {
    let output_format = if command.json {
        OutputFormat::Json
    } else if command.markdown {
        OutputFormat::Markdown
    } else {
        OutputFormat::Text
    };
    let options = ScanOptions {
        profile_name: command.profile.clone(),
        include_hidden: command.hidden,
        include_content: command.content,
        python_worker: command.python_worker,
        max_files: command.max_files.unwrap_or(if command.python_worker {
            100_000
        } else {
            10_000
        }),
        format: output_format,
        fail_on_severity: Some(command.fail_on.as_str().to_string()),
        summary_only: command.summary_only,
        strict_integrations: command.strict_integrations,
        command_name: command_name.to_string(),
    };

    let outcome = match scan_repository(&command.path, &options) {
        Ok(report) => report,
        Err(err) => {
            let has_unknown_profile = err
                .chain()
                .any(|cause| cause.to_string().contains("unknown profile"));
            for (index, cause) in err.chain().enumerate() {
                if index == 0 {
                    eprintln!("error: {cause}");
                } else {
                    eprintln!("  caused by: {cause}");
                }
            }
            if has_unknown_profile {
                let known = available_profiles(Path::new(&command.path));
                if !known.is_empty() {
                    eprintln!("hint: available profiles: {}", known.join(", "));
                    eprintln!("hint: local profiles are loaded from <scan root>/.dn/profiles");
                }
            }
            process::exit(1);
        }
    };

    print_report(&outcome.report, command.json, command.markdown);
    if severity_triggered(&outcome.report, command.fail_on) {
        process::exit(2);
    }
}

fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|err| {
            eprintln!("error: failed to serialize output: {err}");
            process::exit(1);
        })
    );
}

fn run_profile_list(root: &str, json: bool) {
    let names = available_profile_entries(Path::new(root));
    if json {
        let payload: Vec<_> = names
            .into_iter()
            .map(|(name, source)| serde_json::json!({"name": name, "source": source}))
            .collect();
        print_json(&serde_json::json!(payload));
        return;
    }
    for (name, source) in names {
        println!("{name}\t{source}");
    }
}

fn run_profile_show(root: &str, name_or_path: &str, json: bool) {
    match effective_profile(name_or_path, Path::new(root)) {
        Ok((profile, source, diagnostics)) => {
            if json {
                print_json(
                    &serde_json::json!({"source": source, "profile": profile, "diagnostics": diagnostics}),
                );
            } else {
                println!("source={source}");
                println!("name={}", profile.name);
                println!("description={}", profile.description);
                println!("include_hidden={}", profile.include_hidden);
                println!("worker_enabled={}", profile.worker_enabled);
                println!("ai_enabled={}", profile.ai.enabled);
                for diagnostic in diagnostics {
                    println!(
                        "diagnostic={} [{}:{}] {}",
                        diagnostic.level, diagnostic.source, diagnostic.code, diagnostic.message
                    );
                }
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(3);
        }
    }
}

fn run_validate_profile(command: ValidateProfileCommand) {
    let path = PathBuf::from(&command.path);
    if !path.exists() {
        eprintln!("error: profile path does not exist: {}", path.display());
        process::exit(3);
    }
    match effective_profile(&command.path, Path::new(&command.root)) {
        Ok((profile, source, diagnostics)) => {
            if command.json {
                print_json(
                    &serde_json::json!({"valid": true, "source": source, "profile": profile, "diagnostics": diagnostics}),
                );
            } else {
                println!("valid=true");
                println!("source={source}");
                println!("profile={}", profile.name);
                for diagnostic in diagnostics {
                    println!(
                        "diagnostic={} [{}:{}] {}",
                        diagnostic.level, diagnostic.source, diagnostic.code, diagnostic.message
                    );
                }
            }
        }
        Err(err) => {
            if command.json {
                print_json(&serde_json::json!({"valid": false, "error": err.to_string()}));
            } else {
                eprintln!("error: {err}");
            }
            process::exit(3);
        }
    }
}

fn doctor_python_worker(root: &Path) -> Diagnostic {
    let script = root.join("workers/python/dn_worker.py");
    if script.exists() {
        Diagnostic {
            level: "info".to_string(),
            source: "doctor".to_string(),
            code: "python-worker-script-found".to_string(),
            message: format!("found python worker script at {}", script.display()),
            path: Some(script.display().to_string()),
        }
    } else {
        Diagnostic {
            level: "warning".to_string(),
            source: "doctor".to_string(),
            code: "python-worker-script-missing".to_string(),
            message: "python worker script was not found in repository".to_string(),
            path: Some(script.display().to_string()),
        }
    }
}

fn doctor_python_runtime() -> Diagnostic {
    let candidates = ["python3", "python"];
    for candidate in candidates {
        let output = process::Command::new(candidate).arg("--version").output();
        if let Ok(output) = output {
            if output.status.success() {
                let version = String::from_utf8_lossy(if output.stdout.is_empty() {
                    &output.stderr
                } else {
                    &output.stdout
                })
                .trim()
                .to_string();
                return Diagnostic {
                    level: "info".to_string(),
                    source: "doctor".to_string(),
                    code: "python-runtime-found".to_string(),
                    message: format!("found usable Python runtime: {candidate} ({version})"),
                    path: None,
                };
            }
        }
    }

    Diagnostic {
        level: "warning".to_string(),
        source: "doctor".to_string(),
        code: "python-runtime-missing".to_string(),
        message: "python runtime was not found on PATH; python worker cannot start".to_string(),
        path: None,
    }
}

fn doctor_profile_examples(root: &Path) -> Diagnostic {
    let examples_dir = root.join("examples/profiles");
    let count = std::fs::read_dir(&examples_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| matches!(ext, "toml" | "yml" | "yaml"))
        })
        .count();

    if count == 0 {
        Diagnostic {
            level: "warning".to_string(),
            source: "doctor".to_string(),
            code: "profile-examples-missing".to_string(),
            message: "tracked example profiles were not found".to_string(),
            path: Some(examples_dir.display().to_string()),
        }
    } else {
        Diagnostic {
            level: "info".to_string(),
            source: "doctor".to_string(),
            code: "profile-examples-found".to_string(),
            message: format!("found {count} tracked example profile(s) for customization"),
            path: Some(examples_dir.display().to_string()),
        }
    }
}

fn run_doctor(command: DoctorCommand) {
    let root = Path::new(&command.root);
    let mut diagnostics = Vec::new();
    diagnostics.push(doctor_python_worker(root));
    diagnostics.push(doctor_python_runtime());
    diagnostics.push(doctor_profile_examples(root));
    let profile_dir = root.join(".dn/profiles");
    diagnostics.push(Diagnostic {
        level: if profile_dir.exists() {
            "info".to_string()
        } else {
            "warning".to_string()
        },
        source: "doctor".to_string(),
        code: "profile-dir".to_string(),
        message: if profile_dir.exists() {
            "local profile directory is present".to_string()
        } else {
            "local profile directory is absent".to_string()
        },
        path: Some(profile_dir.display().to_string()),
    });
    let has_errors = diagnostics.iter().any(|d| d.level == "error");
    let warnings = diagnostics.iter().filter(|d| d.level == "warning").count();
    if command.json {
        print_json(&serde_json::json!({
            "status": if has_errors { "failing" } else if warnings > 0 { "warning" } else { "ok" },
            "diagnostics": diagnostics
        }));
    } else {
        for diagnostic in &diagnostics {
            println!(
                "{} [{}:{}] {}",
                diagnostic.level, diagnostic.source, diagnostic.code, diagnostic.message
            );
        }
    }
    if has_errors {
        process::exit(3);
    }
}

fn main() {
    let cli = Cli::parse();
    let _ = std::io::stdout().lock().flush();

    match cli.command {
        Commands::Scan(command) => run_scan("scan", command),
        Commands::Review(command) => run_scan("review", command),
        Commands::Profiles(ProfileCommands { command }) => match command {
            ProfileSubcommands::List { json, root } => run_profile_list(&root, json),
            ProfileSubcommands::Show {
                name_or_path,
                json,
                root,
            } => run_profile_show(&root, &name_or_path, json),
        },
        Commands::ValidateProfile(command) => run_validate_profile(command),
        Commands::Doctor(command) => run_doctor(command),
    }
}
