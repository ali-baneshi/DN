#!/usr/bin/env python3
import json
import re
import sys

PROTOCOL_VERSION = "1.0.0"
SENSITIVE_SINKS = [
    r"Runtime\.getRuntime\(\)\.exec",
    r"ProcessBuilder",
    r"Files\.readString",
    r"Files\.writeString",
    r"Paths\.get",
    r"Path\.of",
    r"Statement\.execute",
    r"PreparedStatement",
]


def add_finding(findings, seen, rule, severity, message, category, line):
    key = (rule, line, message)
    if key in seen:
        return
    seen.add(key)
    findings.append({
        "rule": rule,
        "severity": severity,
        "message": message,
        "category": category,
        "line": line,
    })


def analyze(content):
    lines = content.splitlines()
    findings = []
    seen = set()
    tainted = set()

    for index, line in enumerate(lines, start=1):
        lower = line.lower()
        match = re.search(r"(?:String|var)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=.*(?:request\.getParameter|req\.getParameter|System\.getProperty|args\[)", line)
        if match:
            tainted.add(match.group(1))

        if "Runtime.getRuntime().exec" in line or "new ProcessBuilder" in line:
            add_finding(findings, seen, "java-command-exec", "high", "Command execution primitive detected", "security", index)
        if "ObjectInputStream" in line or "XMLDecoder" in line:
            add_finding(findings, seen, "java-dangerous-deserialization", "high", "Unsafe Java deserialization primitive detected", "security", index)
        if "MessageDigest.getInstance" in line and ("MD5" in line or "SHA-1" in line or "SHA1" in line):
            add_finding(findings, seen, "java-weak-hash", "high", "Weak hash primitive detected", "security", index)
        if ("Statement" in line and "execute" in line and "+" in line) or (("SELECT " in line or "INSERT " in line or "UPDATE " in line or "DELETE " in line) and "+" in line):
            add_finding(findings, seen, "java-sql-concatenation", "high", "SQL query appears to be assembled dynamically", "security", index)
        if ("HttpClient.newHttpClient()" in line or ".send(" in line or "openConnection()" in line) and "timeout" not in lower and "setconnecttimeout" not in lower and "setreadtimeout" not in lower:
            add_finding(findings, seen, "java-network-no-timeout", "medium", "Network call lacks an explicit timeout", "reliability", index)
        if re.search(r"catch\s*\([^)]*\)\s*\{\s*\}", line):
            add_finding(findings, seen, "java-empty-catch", "medium", "Empty catch block suppresses failures", "reliability", index)
        if ("Paths.get(" in line or "Path.of(" in line or "new File(" in line) and ("request" in lower or "param" in lower or "filename" in lower):
            add_finding(findings, seen, "java-path-traversal", "high", "Filesystem path uses request-derived input without normalization", "security", index)
        if "assert " in line or "throw new AssertionError" in line:
            add_finding(findings, seen, "java-assertion-leftover", "medium", "Assertion-style failure remains in repository code", "reliability", index)

        for variable in tainted:
            if re.search(rf"\b{re.escape(variable)}\b", line) and any(re.search(sink, line) for sink in SENSITIVE_SINKS):
                add_finding(findings, seen, "java-tainted-flow", "high", f"Request-derived variable {variable} flows into a sensitive sink", "security", index)

    return findings


for raw in sys.stdin:
    try:
        msg = json.loads(raw)
    except Exception:
        continue
    if msg.get("method") == "hello":
        print(json.dumps({
            "protocol_version": PROTOCOL_VERSION,
            "request_id": msg.get("request_id", "hello"),
            "status": "ok",
            "findings": []
        }), flush=True)
    else:
        findings = analyze(msg.get("params", {}).get("content", ""))
        print(json.dumps({
            "protocol_version": PROTOCOL_VERSION,
            "request_id": msg.get("request_id", "analyze"),
            "status": "ok",
            "findings": findings
        }), flush=True)
