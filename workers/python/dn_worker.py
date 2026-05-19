import ast
import json
import sys


def analyze(path, content):
    findings = []

    try:
        tree = ast.parse(content)
    except Exception:
        return findings

    for node in ast.walk(tree):

        if isinstance(node, ast.Call):
            if getattr(node.func, "id", None) == "eval":
                findings.append({
                    "rule": "python-eval-usage",
                    "severity": "high",
                    "message": "Use of eval() detected",
                    "line": node.lineno,
                })

            if getattr(node.func, "id", None) == "print":
                findings.append({
                    "rule": "debug-print",
                    "severity": "low",
                    "message": "print() detected",
                    "line": node.lineno,
                })

        if isinstance(node, ast.Import):
            for name in node.names:
                if name.name == "pdb":
                    findings.append({
                        "rule": "debugger-import",
                        "severity": "medium",
                        "message": "pdb imported",
                        "line": node.lineno,
                    })

    return findings


def respond(payload):
    print(json.dumps(payload), flush=True)


def handle_request(req):
    # New protocol: WorkerRequest from Rust runtime.
    if req.get("method") == "analyze_file" and "params" in req:
        params = req["params"] or {}
        path = params.get("path", "")
        content = params.get("content", "")
        findings = analyze(path, content)
        return {
            "protocol_version": req.get("protocol_version", "0.1.0"),
            "request_id": req.get("request_id", ""),
            "status": "ok",
            "findings": findings,
        }

    # Legacy compatibility for older line based protocol during transition.
    if "path" in req and "content" in req:
        path = req.get("path", "")
        content = req.get("content", "")
        findings = analyze(path, content)
        return {
            "protocol_version": "0.1.0",
            "request_id": "",
            "status": "ok",
            "findings": findings,
        }

    if req.get("method") == "hello":
        return {
            "protocol_version": req.get("protocol_version", "0.1.0"),
            "request_id": req.get("request_id", ""),
            "status": "ok",
            "findings": [],
        }

    return {
        "protocol_version": req.get("protocol_version", "0.1.0"),
        "request_id": req.get("request_id", ""),
        "status": "error",
        "findings": [],
        "error": "unknown request",
    }


for line in sys.stdin:
    line = line.strip()
    if not line:
        continue

    try:
        req = json.loads(line)
    except Exception as err:
        respond({"protocol_version": "0.1.0", "request_id": "", "status": "error", "findings": [], "error": str(err)})
        continue

    response = handle_request(req)
    respond(response)
