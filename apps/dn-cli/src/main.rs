<<<<<<< HEAD
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dn_runtime::{
    apply_safe_fixes, available_profile_entries, available_profiles, effective_profile,
    registered_rule_names, rule_specs, scan_repository, Diagnostic, OutputFormat, ScanOptions,
};
=======
use std::io::Write;
use std::path::Path;
use std::process;

use clap::{Parser, Subcommand};
use dn_runtime::{available_profiles, scan_repository, OutputFormat, ScanOptions};
>>>>>>> feature/persistent-workers

#[derive(Parser, Debug)]
#[command(name = "dn-cli")]
#[command(version)]
#[command(about = "Terminal code review and repository audit assistant")]
<<<<<<< HEAD
=======
#[command(
    long_about = "dn-cli helps you quickly audit unfamiliar, AI-generated, copied, or legacy code.\n\
It scans files through a profile-defined policy and outputs stable JSON or readable Markdown.\n\n\
Examples:\n\
  dn-cli scan . --profile quick\n\
  dn-cli scan . --profile security --json --hidden\n\
  dn-cli review . --profile architecture --markdown\n\
  dn-cli scan . --profile my-security --content\n\
  dn-cli scan . --profile quick --python-worker"
)]
>>>>>>> feature/persistent-workers
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan a repository path and emit a repository review.
<<<<<<< HEAD
    Scan(ScanCommand),
    /// Review is a first-class alias for scan.
    Review(ScanCommand),
    /// Inspect built-in and local profiles.
    Profiles(ProfileCommands),
    /// Validate a profile file.
    ValidateProfile(ValidateProfileCommand),
    /// Run lightweight environment and integration checks.
    Doctor(DoctorCommand),
    /// Apply safe automatic fixes for a subset of deterministic rules.
    Fix(FixCommand),
    /// Inspect the built-in deterministic rule registry.
    Rules(RulesCommand),
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
    #[arg(long)]
    fast: bool,
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

#[derive(Args, Debug, Clone)]
struct FixCommand {
    /// Path to scan and fix
    path: String,
    #[arg(long, default_value = "quick")]
    profile: String,
    #[arg(long)]
    hidden: bool,
    #[arg(long)]
    python_worker: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Debug)]
struct RulesCommand {
    #[arg(long)]
    json: bool,
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
    output.push_str(&format!("- Root: `{}`\n", report.root));
    output.push_str(&format!("- Profile: `{}`\n", report.profile));
    output.push_str(&format!("- Profile source: `{}`\n", report.profile_source));
    output.push_str(&format!("- Worker mode: `{}`\n", report.worker));
    output.push_str(&format!("- Duration: {}ms\n", report.duration_ms));
    output.push_str(&format!("- Truncated: {}\n", report.truncated));
    output.push_str("- Summary only: false\n\n");

    output.push_str("## Integration Status\n\n");
    output.push_str(&format!("- Worker: `{}`\n", report.worker));
    output.push_str(&format!("- Provider: `{}`\n\n", report.provider));

    output.push_str("## Severity Breakdown\n\n");
    output.push_str(&format!(
        "- Findings total: {}\n",
        report
            .files
            .iter()
            .map(|file| file.findings.len())
            .sum::<usize>()
    ));
    output.push_str(&format!(
        "- info={} low={} medium={} high={} critical={}\n\n",
=======
    Scan {
        /// Path to scan (repository root or folder)
        path: String,

        #[arg(long, default_value = "quick")]
        /// Profile name or .dn/profiles/<name>.toml/.yml/.yaml entry
        profile: String,

        /// Print machine-readable JSON output
        #[arg(long, conflicts_with = "markdown")]
        json: bool,

        /// Print markdown output
        #[arg(long, conflicts_with = "json")]
        markdown: bool,

        /// Include up to 1KB content preview for scanned files in JSON/Markdown output
        #[arg(long)]
        content: bool,

        /// Include dotfiles and dot-directories while scanning
        #[arg(long)]
        hidden: bool,

        /// Enable Python worker in addition to profile configuration
        #[arg(long)]
        python_worker: bool,
    },
    /// Review is a first-class alias for scan.
    Review {
        /// Path to review (repository root or folder)
        path: String,

        #[arg(long, default_value = "quick")]
        profile: String,

        /// Print machine-readable JSON output
        #[arg(long, conflicts_with = "markdown")]
        json: bool,

        /// Print markdown output
        #[arg(long, alias = "md", conflicts_with = "json")]
        markdown: bool,

        #[arg(long)]
        content: bool,

        #[arg(long)]
        hidden: bool,

        #[arg(long)]
        python_worker: bool,
    },
}

fn render_markdown(report: &dn_runtime::ScanReport) -> String {
    let mut output = String::new();
    let mut findings_count = 0usize;
    let mut files_with_no_findings = 0usize;
    for file in &report.files {
        findings_count += file.findings.len();
        if file.findings.is_empty() {
            files_with_no_findings += 1;
        }
    }

    let mut files_with_findings = 0usize;

    output.push_str(&format!(
        "# dn-kernel Review Report\n\nProfile: `{}`\n\n",
        report.profile
    ));
    output.push_str(&format!("- Root: `{}`\n", report.root));
    output.push_str(&format!("- Profile source: `{}`\n", report.profile_source));
    output.push_str(&format!("- Provider: `{}`\n", report.provider));
    output.push_str(&format!("- Worker: `{}`\n", report.worker));
    output.push_str(&format!(
        "- Files discovered: {}\n",
        report.files_discovered
    ));
    output.push_str(&format!("- Files analyzed: {}\n", report.files_scanned));
    output.push_str(&format!(
        "- Files returned in report: {}\n",
        report.files_selected
    ));
    output.push_str(&format!("- Skipped files: {}\n", report.files_skipped));
    output.push_str(&format!(
        "- Skipped large files: {}\n",
        report.skipped_large_files
    ));
    output.push_str(&format!("- Total bytes read: {}\n", report.total_bytes));
    output.push_str(&format!(
        "- Truncated by scan limits: {}\n",
        report.truncated
    ));
    output.push_str(&format!("- Duration: {}ms\n\n", report.duration_ms));
    output.push_str(&format!(
        "- Findings: info={} low={} medium={} high={} critical={}\n\n",
>>>>>>> feature/persistent-workers
        report.severity_breakdown.info,
        report.severity_breakdown.low,
        report.severity_breakdown.medium,
        report.severity_breakdown.high,
        report.severity_breakdown.critical
    ));
<<<<<<< HEAD

    output.push_str("## Findings\n\n");
    let mut found_any = false;
=======
    output.push_str(&format!(
        "## Summary\n\n- Total findings: {}\n- Files with findings: {}\n- Files reported without findings: {}\n\n",
        findings_count,
        report.files.len().saturating_sub(files_with_no_findings),
        files_with_no_findings
    ));

    output.push_str("## Top findings\n\n");
>>>>>>> feature/persistent-workers
    for file in &report.files {
        if file.findings.is_empty() {
            continue;
        }
<<<<<<< HEAD
        found_any = true;
        output.push_str(&format!("### `{}`\n", file.path));
        for finding in &file.findings {
            let line_suffix = finding
                .line
                .map(|line| format!(" line {}", line))
                .unwrap_or_default();
            output.push_str(&format!(
                "- **{}** [{} / {}{}] {}\n",
                finding.severity,
                finding.source.as_deref().unwrap_or("local"),
                finding.rule,
                line_suffix,
=======
        files_with_findings += 1;
        output.push_str(&format!("### `{}`\n", file.path));
        for finding in &file.findings {
            output.push_str(&format!(
                "- **{}** [{}] {}: {}\n",
                finding.severity,
                finding.rule,
                finding.source.clone().unwrap_or_else(|| "scan".to_string()),
>>>>>>> feature/persistent-workers
                finding.message
            ));
        }
        output.push('\n');
    }
<<<<<<< HEAD
    if !found_any {
        output.push_str("*No findings were reported for the current profile.*\n\n");
    }

    output.push_str("## Diagnostics\n\n");
    output.push_str("- No diagnostics reported.\n");
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

    println!("schema_version=2");
    println!("root={}", report.root);
    println!("profile={}", report.profile);
    println!("profile_source={}", report.profile_source);
    println!("command={}", report.worker);
    println!("worker={}", report.worker);
=======
    if files_with_findings == 0 {
        output.push_str("*No findings were reported for the current profile.*\n\n");
    }

    if !report.errors.is_empty() {
        output.push_str("## Errors\n\n");
        output.push_str(&format!("- Count: {}\n", report.errors.len()));
        for error in &report.errors {
            output.push_str(&format!("- {}\n", error));
        }
    }

    output
}

fn print_report(report: dn_runtime::ScanReport, want_json: bool, want_markdown: bool) {
    if want_json {
        match serde_json::to_string_pretty(&report) {
            Ok(payload) => println!("{payload}"),
            Err(err) => {
                eprintln!("error: failed to serialize report: {err}");
                process::exit(1);
            }
        }
        return;
    }

    if want_markdown {
        print!("{}", render_markdown(&report));
        return;
    }

    let mut findings_count = 0usize;
    for file in &report.files {
        findings_count += file.findings.len();
    }

    println!("root={}", report.root);
    println!("profile={}", report.profile);
    println!("profile_source={}", report.profile_source);
>>>>>>> feature/persistent-workers
    println!("provider={}", report.provider);
    println!("files_discovered={}", report.files_discovered);
    println!("files_scanned={}", report.files_scanned);
    println!("files_selected={}", report.files_selected);
<<<<<<< HEAD
    println!(
        "findings={}",
        report
            .files
            .iter()
            .map(|file| file.findings.len())
            .sum::<usize>()
    );
    println!("summary={}", report.summary);
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

fn run_scan(_command_name: &str, command: ScanCommand) {
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
        summary_only: command.summary_only,
        fast: command.fast,
        format: output_format,
    };

    let outcome = match scan_repository(&command.path, &options) {
=======
    println!("files_skipped={}", report.files_skipped);
    println!("skipped_large_files={}", report.skipped_large_files);
    println!("total_files={}", report.total_files);
    println!("bytes={}", report.total_bytes);
    println!("truncated={}", report.truncated);
    println!("duration_ms={}", report.duration_ms);
    println!("findings={}", findings_count);
    println!(
        "severity=info:{} low:{} medium:{} high:{} critical:{}",
        report.severity_breakdown.info,
        report.severity_breakdown.low,
        report.severity_breakdown.medium,
        report.severity_breakdown.high,
        report.severity_breakdown.critical
    );
    if !report.errors.is_empty() {
        println!("errors:");
        for error in &report.errors {
            println!("  - {}", error);
        }
    }
}

fn run_scan(
    path: String,
    profile: String,
    output_path: String,
    content: bool,
    hidden: bool,
    python_worker: bool,
    json: bool,
    markdown: bool,
) {
    let options = ScanOptions {
        profile_name: profile,
        include_hidden: hidden,
        include_content: content,
        python_worker,
        max_files: if python_worker { 100_000 } else { 10_000 },
        format: if json {
            OutputFormat::Json
        } else if markdown {
            OutputFormat::Markdown
        } else {
            OutputFormat::Text
        },
    };

    let report = match scan_repository(&path, &options) {
>>>>>>> feature/persistent-workers
        Ok(report) => report,
        Err(err) => {
            let has_unknown_profile = err
                .chain()
                .any(|cause| cause.to_string().contains("unknown profile"));
<<<<<<< HEAD
            for (index, cause) in err.chain().enumerate() {
                if index == 0 {
                    eprintln!("error: {cause}");
=======

            let mut first = true;
            for cause in err.chain() {
                if first {
                    eprintln!("error: {cause}");
                    first = false;
>>>>>>> feature/persistent-workers
                } else {
                    eprintln!("  caused by: {cause}");
                }
            }
            if has_unknown_profile {
<<<<<<< HEAD
                let known = available_profiles(Path::new(&command.path));
=======
                let known = maybe_suggest_profile(&output_path);
>>>>>>> feature/persistent-workers
                if !known.is_empty() {
                    eprintln!("hint: available profiles: {}", known.join(", "));
                    eprintln!("hint: local profiles are loaded from <scan root>/.dn/profiles");
                }
            }
            process::exit(1);
        }
    };
<<<<<<< HEAD

    print_report(&outcome, command.json, command.markdown);
    if severity_triggered(&outcome, command.fail_on) {
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

fn run_rules(json: bool) {
    if json {
        let payload: Vec<_> = rule_specs()
            .iter()
            .map(|rule| {
                serde_json::json!({
                    "name": rule.name,
                    "severity": rule.severity,
                    "category": rule.category,
                    "summary": rule.summary,
                    "supports_fix": rule.supports_fix
                })
            })
            .collect();
        print_json(&serde_json::json!(payload));
        return;
    }

    for rule in rule_specs() {
        println!(
            "{}	{}	{}	fix={}	{}",
            rule.name, rule.severity, rule.category, rule.supports_fix, rule.summary
        );
    }
}

fn run_fix(command: FixCommand) {
    let options = ScanOptions {
        profile_name: command.profile.clone(),
        include_hidden: command.hidden,
        include_content: false,
        python_worker: command.python_worker,
        max_files: 10_000,
        summary_only: false,
        fast: true,
        format: OutputFormat::Json,
    };

    let outcome = match scan_repository(&command.path, &options) {
        Ok(report) => report,
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    };

    let root = Path::new(&command.path);
    let fixable = [
        "todo-comment",
        "debug-print",
        "commented-out-code",
        "wildcard-import",
    ];
    let mut applied = Vec::new();

    let mut files_to_fix = Vec::new();
    for file in &outcome.files {
        let abs = root.join(&file.path);
        if let Ok(original) = fs::read_to_string(&abs) {
            let matches = dn_runtime::rules::analyze_registered_rules(&file.path, None, &original);
            if matches
                .iter()
                .any(|rule_match| fixable.contains(&rule_match.finding.rule.as_str()))
            {
                files_to_fix.push((file.path.clone(), original, matches));
            }
        }
    }

    for (path, original, matches) in files_to_fix {
        let target_fixes: Vec<_> = matches
            .into_iter()
            .filter(|finding| fixable.contains(&finding.finding.rule.as_str()))
            .filter_map(|finding| {
                let line = finding.finding.line?;
                let replacement = match finding.finding.rule.as_str() {
                    "todo-comment" | "debug-print" | "commented-out-code" => String::new(),
                    "wildcard-import" => format!(
                        "// REVIEW: replace wildcard import with explicit imports (line {})",
                        line
                    ),
                    _ => return None,
                };
                Some(dn_runtime::rules::RuleFix {
                    line,
                    replacement,
                    description: format!("Auto-fix {}", finding.finding.rule),
                })
            })
            .collect();

        if target_fixes.is_empty() {
            continue;
        }

        let fixed = apply_safe_fixes(&original, &target_fixes);
        if !command.dry_run {
            if let Err(err) = fs::write(root.join(&path), fixed.as_bytes()) {
                eprintln!(
                    "error: failed to write {}: {err}",
                    root.join(&path).display()
                );
                process::exit(1);
            }
        }
        applied.push(serde_json::json!({
            "path": path,
            "fixes": target_fixes.len()
        }));
    }

    if command.json {
        print_json(&serde_json::json!({
            "applied": applied,
            "dry_run": command.dry_run,
            "fixable_rules": registered_rule_names().into_iter().filter(|name| fixable.contains(name)).collect::<Vec<_>>()
        }));
    } else {
        for item in &applied {
            println!(
                "fixed={} count={}",
                item["path"].as_str().unwrap_or(""),
                item["fixes"].as_u64().unwrap_or(0)
            );
        }
        if command.dry_run {
            println!("dry_run=true");
        }
    }
=======
    print_report(report, json, markdown);
}

fn maybe_suggest_profile(path: impl AsRef<Path>) -> Vec<String> {
    available_profiles(path.as_ref())
>>>>>>> feature/persistent-workers
}

fn main() {
    let cli = Cli::parse();
<<<<<<< HEAD
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
        Commands::Fix(command) => run_fix(command),
        Commands::Rules(command) => run_rules(command.json),
=======

    match cli.command {
        Commands::Scan {
            path,
            profile,
            json,
            markdown,
            content,
            hidden,
            python_worker,
        }
        | Commands::Review {
            path,
            profile,
            json,
            markdown,
            content,
            hidden,
            python_worker,
        } => {
            let _ = std::io::stdout().lock().flush();
            let root_for_hint = path.clone();
            run_scan(
                path,
                profile,
                root_for_hint,
                content,
                hidden,
                python_worker,
                json,
                markdown,
            );
        }
>>>>>>> feature/persistent-workers
    }
}
