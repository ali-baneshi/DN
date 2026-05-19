import ast
import json
import sys


PROTOCOL_VERSION = "1.0.0"


def analyze(content):
    findings = []
    try:
        tree = ast.parse(content)
    except Exception:
        return findings

    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            if getattr(node.func, "id", None) == "eval":
                findings.append(
                    {
                        "severity": "high",
                        "rule": "python-eval-usage",
                        "message": "Use of eval() detected",
                        "line": node.lineno,
                    }
                )

            if getattr(node.func, "id", None) == "print":
                findings.append(
                    {
                        "severity": "low",
                        "rule": "debug-print",
                        "message": "print() detected",
                        "line": node.lineno,
                    }
                )

        if isinstance(node, ast.Import):
            for name in node.names:
                if name.name == "pdb":
                    findings.append(
                        {
                            "severity": "medium",
                            "rule": "debugger-import",
                            "message": "pdb imported",
                            "line": node.lineno,
                        }
                    )

    return findings


def respond(payload):
    print(json.dumps(payload), flush=True)


def handle_request(req):
    if req.get("method") == "hello":
        return {
            "protocol_version": req.get("protocol_version", PROTOCOL_VERSION),
            "request_id": req.get("request_id", ""),
            "status": "ok",
            "findings": [],
        }

    if req.get("method") == "analyze_file" and "params" in req:
        params = req.get("params") or {}
        path = params.get("path", "")
        content = params.get("content", "")
        return {
            "protocol_version": req.get("protocol_version", PROTOCOL_VERSION),
            "request_id": req.get("request_id", ""),
            "status": "ok",
            "findings": analyze(content),
            "path": path,
        }

    return {
        "protocol_version": req.get("protocol_version", PROTOCOL_VERSION),
        "request_id": req.get("request_id", ""),
        "status": "error",
        "error": "unknown request",
        "findings": [],
    }


def main():
    for raw in sys.stdin:
        raw = raw.strip()
        if not raw:
            continue
        try:
            req = json.loads(raw)
            respond(handle_request(req))
        except Exception as err:
            respond(
                {
                    "protocol_version": PROTOCOL_VERSION,
                    "request_id": "",
                    "status": "error",
                    "findings": [],
                    "error": str(err),
                }
            )


if __name__ == "__main__":
    main()
