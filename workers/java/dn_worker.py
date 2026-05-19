#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    try:
        msg = json.loads(line)
    except Exception:
        continue
    if msg.get('method') == 'hello':
        print(json.dumps({"protocol_version": "0.1.0", "worker_name": "dn-java-worker", "capabilities": ["java"]}), flush=True)
    else:
        findings = []
        if 'Runtime.getRuntime().exec' in msg.get('content', ''):
            findings.append({"rule": "java-runtime-exec", "severity": "high", "message": "Runtime.exec usage detected", "category": "security", "line": 1})
        print(json.dumps({"protocol_version": "0.1.0", "findings": findings}), flush=True)
