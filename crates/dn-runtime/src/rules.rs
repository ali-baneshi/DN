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
    RuleSpec {
        name: "empty-error-handler",
        severity: "medium",
        category: "reliability",
        summary: "Error handling block is empty or only suppresses the failure",
        supports_fix: false,
    },
    RuleSpec {
        name: "weak-hash-usage",
        severity: "high",
        category: "security",
        summary: "Weak hash primitive detected",
        supports_fix: false,
    },
    RuleSpec {
        name: "insecure-random",
        severity: "high",
        category: "security",
        summary: "Non-cryptographic randomness used in security-sensitive context",
        supports_fix: false,
    },
    RuleSpec {
        name: "shell-command-concatenation",
        severity: "high",
        category: "security",
        summary: "Shell command is assembled via string concatenation or interpolation",
        supports_fix: false,
    },
    RuleSpec {
        name: "network-without-timeout",
        severity: "medium",
        category: "reliability",
        summary: "Network client call is made without an explicit timeout",
        supports_fix: false,
    },
    RuleSpec {
        name: "sql-string-concatenation",
        severity: "high",
        category: "security",
        summary: "SQL query appears to be assembled via string concatenation",
        supports_fix: false,
    },
    RuleSpec {
        name: "path-traversal-join",
        severity: "high",
        category: "security",
        summary: "Path is built from user-controlled input without normalization",
        supports_fix: false,
    },
    RuleSpec {
        name: "dangerous-deserialization",
        severity: "high",
        category: "security",
        summary: "Unsafe deserialization primitive detected",
        supports_fix: false,
    },
    RuleSpec {
        name: "assert-or-panic-in-production",
        severity: "medium",
        category: "reliability",
        summary: "Assertion or panic-style failure remains in production code paths",
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

fn contains_any(haystack: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| haystack.contains(pattern))
}

fn looks_like_empty_block(lines: &[&str], start: usize) -> bool {
    let mut body_started = false;
    for line in lines.iter().skip(start).take(5) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !body_started {
            if trimmed.contains("{}") {
                return true;
            }
            if trimmed.contains('{') {
                let tail = trimmed
                    .split_once('{')
                    .map(|(_, tail)| tail.trim())
                    .unwrap_or("");
                if tail == "}"
                    || tail == "};"
                    || tail == "catch {}"
                    || tail == "catch(err) {}"
                    || tail == "catch (err) {}"
                {
                    return true;
                }
                body_started = true;
                if tail.is_empty() {
                    continue;
                }
                return false;
            }
            if trimmed.ends_with(':') {
                body_started = true;
                continue;
            }
            continue;
        }
        if trimmed == "}"
            || trimmed == "pass"
            || trimmed == "// ignored"
            || trimmed == "// ignore"
            || trimmed == "// noop"
            || trimmed == "/* ignored */"
        {
            return true;
        }
        if trimmed.starts_with("except") || trimmed.starts_with("catch") {
            continue;
        }
        return false;
    }
    false
}

fn detect_empty_error_handlers(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_handler = match language {
            "javascript" | "typescript" => {
                trimmed.starts_with("catch")
                    || trimmed.contains(".catch(() =>")
                    || trimmed.contains(".catch((")
                    || trimmed.contains("catch (")
            }
            "java" => {
                trimmed.starts_with("catch ")
                    || trimmed.starts_with("catch(")
                    || trimmed.contains("catch (")
            }
            "python" => trimmed.starts_with("except") || trimmed == "except:",
            "rust" => trimmed.contains("if let Err(") || trimmed.contains("match "),
            _ => false,
        };
        let inline_empty = trimmed.contains("{}") || trimmed.ends_with("{ }");
        if is_handler && (inline_empty || looks_like_empty_block(lines, idx)) {
            push(
                matches,
                "empty-error-handler",
                "medium",
                "Error handling block is empty and suppresses failures".to_string(),
                "reliability",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_weak_hash_usage(lines: &[&str], matches: &mut Vec<RuleMatch>) {
    let patterns = [
        "md5",
        "sha1",
        r#"messagedigest.getinstance("md5")"#,
        "crypto.createhash('md5'",
        r#"crypto.createhash("md5""#,
    ];
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if contains_any(&lower, &patterns) {
            push(
                matches,
                "weak-hash-usage",
                "high",
                "Weak hash primitive detected; prefer a modern password hashing or SHA-256+ primitive".to_string(),
                "security",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_insecure_random(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let matched = match language {
            "javascript" | "typescript" => {
                lower.contains("math.random()")
                    && (lower.contains("token")
                        || lower.contains("secret")
                        || lower.contains("password")
                        || lower.contains("key"))
            }
            "python" => {
                lower.contains("random.random(")
                    || lower.contains("random.choice(")
                        && (lower.contains("token") || lower.contains("secret"))
            }
            "java" => {
                lower.contains("new random(")
                    && (lower.contains("token")
                        || lower.contains("secret")
                        || lower.contains("password"))
            }
            _ => false,
        };
        if matched {
            push(
                matches,
                "insecure-random",
                "high",
                "Non-cryptographic randomness appears to generate a sensitive value".to_string(),
                "security",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_shell_command_concat(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let matched = match language {
            "javascript" | "typescript" => {
                (trimmed.contains("exec(") || trimmed.contains("spawn("))
                    && (trimmed.contains("+") || trimmed.contains("${"))
            }
            "python" => {
                trimmed.contains("subprocess")
                    && (trimmed.contains("+")
                        || trimmed.contains("%s")
                        || trimmed.contains(".format("))
            }
            "java" => {
                trimmed.contains("exec(")
                    && (trimmed.contains("+") || trimmed.contains("String.format("))
            }
            _ => false,
        };
        if matched {
            push(
                matches,
                "shell-command-concatenation",
                "high",
                "Shell command is constructed dynamically; prefer argument arrays or strict escaping".to_string(),
                "security",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_network_without_timeout(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let matched = match language {
            "javascript" | "typescript" => {
                (lower.contains("fetch(")
                    || lower.contains("axios.get(")
                    || lower.contains("axios.post("))
                    && !lower.contains("signal")
                    && !lower.contains("timeout")
            }
            "python" => {
                (lower.contains("requests.get(") || lower.contains("requests.post("))
                    && !lower.contains("timeout=")
            }
            "java" => {
                lower.contains("httpclient.newhttpclient(")
                    || (lower.contains("url.openconnection(")
                        && !lower.contains("setconnecttimeout")
                        && !lower.contains("setreadtimeout"))
            }
            _ => false,
        };
        if matched {
            push(
                matches,
                "network-without-timeout",
                "medium",
                "Network call lacks an explicit timeout and may hang indefinitely".to_string(),
                "reliability",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_sql_string_concat(lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if (lower.contains("select ")
            || lower.contains("insert ")
            || lower.contains("update ")
            || lower.contains("delete "))
            && (line.contains('+') || line.contains("${") || line.contains(".format("))
        {
            push(
                matches,
                "sql-string-concatenation",
                "high",
                "SQL query appears to be assembled dynamically; prefer parameterized queries"
                    .to_string(),
                "security",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_path_traversal_join(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let matched = match language {
            "javascript" | "typescript" => {
                (lower.contains("path.join(") || lower.contains("path.resolve("))
                    && (lower.contains("req.")
                        || lower.contains("params")
                        || lower.contains("query")
                        || lower.contains("body"))
            }
            "python" => {
                (lower.contains("os.path.join(") || lower.contains("pathlib.path("))
                    && (lower.contains("request")
                        || lower.contains("user_input")
                        || lower.contains("filename"))
            }
            "java" => {
                (lower.contains("paths.get(") || lower.contains("path.of("))
                    && (lower.contains("request")
                        || lower.contains("param")
                        || lower.contains("filename"))
            }
            _ => false,
        };
        if matched {
            push(
                matches,
                "path-traversal-join",
                "high",
                "Path uses request-derived input; validate and normalize before filesystem access"
                    .to_string(),
                "security",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_dangerous_deserialization(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let matched = match language {
            "python" => lower.contains("pickle.loads(") || lower.contains("yaml.load("),
            "javascript" | "typescript" => lower.contains("deserialize(") && lower.contains("user"),
            "java" => lower.contains("objectinputstream") || lower.contains("xmldecoder"),
            _ => false,
        };
        if matched {
            push(
                matches,
                "dangerous-deserialization",
                "high",
                "Unsafe deserialization primitive detected on potentially untrusted data"
                    .to_string(),
                "security",
                Some((idx + 1) as u32),
                None,
            );
        }
    }
}

fn detect_assert_or_panic(language: &str, lines: &[&str], matches: &mut Vec<RuleMatch>) {
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let matched = match language {
            "rust" => trimmed.contains("panic!(") || trimmed.contains("assert!("),
            "python" => trimmed.starts_with("assert "),
            "javascript" | "typescript" => {
                trimmed.contains("console.assert(")
                    || trimmed.contains("throw new Error(") && trimmed.contains("TODO")
            }
            "java" => {
                trimmed.starts_with("assert ") || trimmed.contains("throw new AssertionError(")
            }
            _ => false,
        };
        if matched {
            push(
                matches,
                "assert-or-panic-in-production",
                "medium",
                "Assertion-style failure remains in repository code; confirm it is intended for production".to_string(),
                "reliability",
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
    detect_empty_error_handlers(language, &lines, &mut matches);
    detect_weak_hash_usage(&lines, &mut matches);
    detect_insecure_random(language, &lines, &mut matches);
    detect_shell_command_concat(language, &lines, &mut matches);
    detect_network_without_timeout(language, &lines, &mut matches);
    detect_sql_string_concat(&lines, &mut matches);
    detect_path_traversal_join(language, &lines, &mut matches);
    detect_dangerous_deserialization(language, &lines, &mut matches);
    detect_assert_or_panic(language, &lines, &mut matches);
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

#[cfg(test)]
mod tests {
    use super::{analyze_registered_rules, registered_rule_names};

    fn rules_for(path: &str, language: Option<&str>, content: &str) -> Vec<String> {
        analyze_registered_rules(path, language, content)
            .into_iter()
            .map(|rule_match| rule_match.finding.rule)
            .collect()
    }

    #[test]
    fn registry_has_expanded_multilanguage_coverage() {
        assert!(registered_rule_names().len() >= 19);
    }

    #[test]
    fn typescript_rules_cover_common_security_and_reliability_patterns() {
        let findings = rules_for(
            "src/app.ts",
            Some("typescript"),
            r#"
            import { exec } from 'child_process';
            async function run(userInput: string) {
                try {
                    await fetch(url);
                } catch (err) {}
                const sql = "SELECT * FROM users WHERE id = " + userInput;
                exec(`cat ${userInput}`);
                const token = Math.random().toString();
                return path.join(baseDir, req.params.file);
            }
            "#,
        );

        for expected in [
            "empty-error-handler",
            "network-without-timeout",
            "sql-string-concatenation",
            "shell-command-concatenation",
            "insecure-random",
            "path-traversal-join",
        ] {
            assert!(
                findings.iter().any(|rule| rule == expected),
                "missing {expected}"
            );
        }
    }

    #[test]
    fn java_and_python_rules_cover_additional_security_smells() {
        let java_findings = rules_for(
            "src/Main.java",
            Some("java"),
            r#"
            import java.io.ObjectInputStream;
            class Main {
                void load(String filename, String token) throws Exception {
                    try {
                        doRisky();
                    } catch (Exception ex) {}
                    String sql = "SELECT * FROM accounts WHERE id = " + filename;
                    Runtime.getRuntime().exec("sh -c " + filename);
                    String hash = MessageDigest.getInstance("MD5").digest(token.getBytes()).toString();
                    Path path = Paths.get(baseDir, filename);
                }
            }
            "#,
        );
        for expected in [
            "empty-error-handler",
            "sql-string-concatenation",
            "shell-command-concatenation",
            "weak-hash-usage",
            "path-traversal-join",
            "dangerous-deserialization",
        ] {
            assert!(
                java_findings.iter().any(|rule| rule == expected),
                "missing {expected}"
            );
        }

        let python_findings = rules_for(
            "worker.py",
            Some("python"),
            r#"
            import pickle
            import requests
            def load(user_input):
                try:
                    run()
                except Exception:
                    pass
                data = pickle.loads(user_input)
                requests.get("https://example.com")
                query = "SELECT * FROM users WHERE id = " + user_input
                return data
            "#,
        );
        for expected in [
            "dangerous-deserialization",
            "network-without-timeout",
            "sql-string-concatenation",
        ] {
            assert!(
                python_findings.iter().any(|rule| rule == expected),
                "missing {expected}"
            );
        }
    }
}
