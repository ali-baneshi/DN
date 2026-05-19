use crate::Finding;

#[derive(Debug, Clone)]
pub struct RuleFix {
    pub line: u32,
    pub replacement: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub finding: Finding,
    pub fix: Option<RuleFix>,
}

#[derive(Debug, Clone, Copy)]
pub struct RuleSpec {
    pub name: &'static str,
    pub severity: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
    pub supports_fix: bool,
}

pub const RULE_REGISTRY: &[RuleSpec] = &[
    RuleSpec {
        name: "todo-comment",
        severity: "info",
        category: "maintainability",
        summary: "TODO marker left in comments",
        supports_fix: true,
    },
    RuleSpec {
        name: "unsafe-usage",
        severity: "high",
        category: "safety",
        summary: "Unsafe language construct present in code",
        supports_fix: false,
    },
    RuleSpec {
        name: "possible-secret",
        severity: "high",
        category: "security",
        summary: "Secret-like assignment detected",
        supports_fix: false,
    },
    RuleSpec {
        name: "hardcoded-value",
        severity: "high",
        category: "security",
        summary: "Hardcoded credential-like literal detected",
        supports_fix: false,
    },
    RuleSpec {
        name: "large-file",
        severity: "low",
        category: "maintainability",
        summary: "Large file may hide complexity",
        supports_fix: false,
    },
    RuleSpec {
        name: "deprecated-api",
        severity: "medium",
        category: "modernization",
        summary: "Deprecated API markers or imports detected",
        supports_fix: false,
    },
    RuleSpec {
        name: "hard-to-read-function",
        severity: "medium",
        category: "maintainability",
        summary: "Function body exceeds simple readability limits",
        supports_fix: false,
    },
    RuleSpec {
        name: "debug-print",
        severity: "low",
        category: "maintainability",
        summary: "Debug print statement left in repository code",
        supports_fix: true,
    },
    RuleSpec {
        name: "commented-out-code",
        severity: "low",
        category: "maintainability",
        summary: "Comment appears to contain disabled code",
        supports_fix: false,
    },
    RuleSpec {
        name: "wildcard-import",
        severity: "medium",
        category: "maintainability",
        summary: "Wildcard import detected",
        supports_fix: false,
    },
];

fn push(
    matches: &mut Vec<RuleMatch>,
    rule: &str,
    severity: &str,
    message: String,
    category: &str,
    line: Option<u32>,
    fix: Option<RuleFix>,
) {
    matches.push(RuleMatch {
        finding: Finding {
            rule: rule.to_string(),
            severity: severity.to_string(),
            message,
            category: Some(category.to_string()),
            line,
            source: Some("rule-registry".to_string()),
            origin: "deterministic".to_string(),
        },
        fix,
    });
}

fn detect_comment_code_smell(lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*');
        if is_comment
            && (trimmed.contains("if (")
                || trimmed.contains("return ")
                || trimmed.contains("let ")
                || trimmed.contains("const ")
                || trimmed.contains("public ")
                || trimmed.contains("private ")
                || trimmed.contains(';'))
        {
            push(
                matches,
                "commented-out-code",
                "low",
                "Comment appears to contain disabled code".to_string(),
                "maintainability",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_debug_prints(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    let patterns = match language {
        "rust" => &["dbg!", "println!"][..],
        "python" => &["print("][..],
        "javascript" | "typescript" => &["console.log(", "console.debug("][..],
        "java" => &["System.out.println("][..],
        _ => &[][..],
    };
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if patterns.iter().any(|p| trimmed.starts_with(p)) {
            let fix = Some(RuleFix {
                line: (idx + 1) as u32,
                replacement: String::new(),
                description: "Remove leftover debug print statement".to_string(),
            });
            push(
                matches,
                "debug-print",
                "low",
                "Debug print statement left in code".to_string(),
                "maintainability",
                Some((idx + 1) as u32),
                fix,
            );
        }
    }
}

fn detect_wildcard_imports(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let matched = match language {
            "python" => trimmed.starts_with("from ") && trimmed.contains(" import *"),
            "java" => trimmed.starts_with("import ") && trimmed.ends_with(".*;"),
            "javascript" | "typescript" => trimmed.starts_with("import * as "),
            _ => false,
        };
        if matched {
            push(
                matches,
                "wildcard-import",
                "medium",
                "Wildcard import detected".to_string(),
                "maintainability",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_deprecated_markers(lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if lower.contains("deprecated") {
            push(
                matches,
                "deprecated-api",
                "medium",
                "Deprecated marker or API usage detected".to_string(),
                "modernization",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_long_functions(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    let start_markers = match language {
        "rust" => &["fn ", "pub fn "][..],
        "python" => &["def "][..],
        "javascript" | "typescript" => &["function ", "const ", "let ", "class "][..],
        "java" => &["public ", "private ", "protected "][..],
        _ => &[][..],
    };
    let mut start: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if start.is_none()
            && start_markers
                .iter()
                .any(|marker| trimmed.starts_with(marker))
            && trimmed.contains('(')
        {
            start = Some(idx);
            continue;
        }
        if let Some(begin) = start {
            let span = idx.saturating_sub(begin) + 1;
            if trimmed == "}"
                || trimmed == "end"
                || (language == "python"
                    && !line.starts_with(' ')
                    && !line.starts_with('\t')
                    && !trimmed.is_empty())
            {
                if span > 40 {
                    push(
                        matches,
                        "hard-to-read-function",
                        "medium",
                        format!("Function spans {} lines and may be hard to review", span),
                        "maintainability",
                        Some((begin + 1) as u32),
                        None,
                    );
                }
                start = None;
            }
        }
    }
}

pub fn registered_rule_names() -> Vec<&'static str> {
    RULE_REGISTRY.iter().map(|rule| rule.name).collect()
}

pub fn rule_specs() -> &'static [RuleSpec] {
    RULE_REGISTRY
}

pub fn analyze_registered_rules(
    path: &str,
    language: Option<&str>,
    content: &str,
) -> Vec<RuleMatch> {
    let ext_language = language
        .or_else(|| {
            std::path::Path::new(path)
                .extension()
                .and_then(|ext| ext.to_str())
        })
        .map(|lang| match lang {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "java" => "java",
            other => other,
        });

    let language = match ext_language {
        Some(language) => language,
        None => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut matches = Vec::new();
    detect_comment_code_smell(&lines, &mut matches);
    detect_debug_prints(language, &lines, &mut matches);
    detect_wildcard_imports(language, &lines, &mut matches);
    detect_deprecated_markers(&lines, &mut matches);
    detect_long_functions(language, &lines, &mut matches);
    matches
}

pub fn apply_safe_fixes(content: &str, fixes: &[RuleFix]) -> String {
    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let mut ordered = fixes.to_vec();
    ordered.sort_by(|a, b| b.line.cmp(&a.line));
    for fix in ordered {
        let index = fix.line.saturating_sub(1) as usize;
        if index < lines.len() {
            if fix.replacement.is_empty() {
                lines.remove(index);
            } else {
                lines[index] = fix.replacement;
            }
        }
    }
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}
