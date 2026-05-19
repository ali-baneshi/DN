#!/usr/bin/env node
const readline = require('readline');

const PROTOCOL_VERSION = '1.0.0';

function addFinding(findings, seen, rule, severity, message, category, line) {
  const key = `${rule}:${line}:${message}`;
  if (seen.has(key)) return;
  seen.add(key);
  findings.push({ rule, severity, message, category, line });
}

function analyze(content) {
  const lines = content.split(/\r?\n/);
  const findings = [];
  const seen = new Set();
  const taintedVars = new Set();

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    const lower = line.toLowerCase();

    const taintMatch = line.match(/(?:const|let|var)\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=.*(?:req\.(?:body|query|params)|context\.request|location\.search|process\.argv)/);
    if (taintMatch) taintedVars.add(taintMatch[1]);

    if (/\beval\s*\(/.test(line)) {
      addFinding(findings, seen, 'ts-eval-usage', 'high', 'eval() usage detected; avoid executing dynamic code', 'security', i + 1);
    }
    if (/(new Function\s*\()|(setTimeout\s*\(\s*["'])|(setInterval\s*\(\s*["'])/.test(line)) {
      addFinding(findings, seen, 'ts-dynamic-code', 'high', 'Dynamic code execution primitive detected', 'security', i + 1);
    }
    if (/innerHTML\s*=/.test(line) && !/DOMPurify|sanitize/i.test(line)) {
      addFinding(findings, seen, 'ts-dom-xss', 'high', 'innerHTML assignment without obvious sanitization', 'security', i + 1);
    }
    if (/(child_process|exec\(|spawn\(|execSync\()/.test(line) && (line.includes('+') || line.includes('${'))) {
      addFinding(findings, seen, 'ts-command-injection', 'high', 'Shell command appears to be built from dynamic input', 'security', i + 1);
    }
    if (/(fs\.readFile|fs\.writeFile|fs\.createReadStream|path\.join|path\.resolve)/.test(line) && /(req\.|params|query|body)/.test(line)) {
      addFinding(findings, seen, 'ts-path-traversal', 'high', 'Filesystem path uses request-derived input without normalization', 'security', i + 1);
    }
    if (/fetch\(/.test(line) && !/signal|timeout/i.test(line)) {
      addFinding(findings, seen, 'ts-network-no-timeout', 'medium', 'fetch() call lacks an explicit timeout/cancellation path', 'reliability', i + 1);
    }
    if (/catch\s*\([^)]*\)\s*\{\s*\}/.test(line)) {
      addFinding(findings, seen, 'ts-empty-catch', 'medium', 'Empty catch block suppresses failures', 'reliability', i + 1);
    }
    if (/jwt\.sign\(/.test(line) && /['"]none['"]/.test(line)) {
      addFinding(findings, seen, 'ts-jwt-none-alg', 'high', 'JWT signing appears to allow the none algorithm', 'security', i + 1);
    }
    if (/crypto\.createHash\(/i.test(line) && /md5|sha1/i.test(line)) {
      addFinding(findings, seen, 'ts-weak-hash', 'high', 'Weak hash primitive detected', 'security', i + 1);
    }

    for (const variable of taintedVars) {
      const taintedPattern = new RegExp(`\\b${variable}\\b`);
      if (taintedPattern.test(line) && /(?:exec\(|spawn\(|fetch\(|axios\.|innerHTML\s*=|path\.join\(|path\.resolve\()/i.test(line)) {
        addFinding(findings, seen, 'ts-tainted-flow', 'high', `Request-derived variable ${variable} flows into a sensitive sink`, 'security', i + 1);
      }
    }
  }

  return findings;
}

const rl = readline.createInterface({ input: process.stdin, terminal: false });
let buffer = '';
rl.on('line', (line) => {
  buffer += line;
  try {
    const message = JSON.parse(buffer);
    buffer = '';

    if (message.method === 'hello') {
      console.log(JSON.stringify({
        protocol_version: PROTOCOL_VERSION,
        request_id: message.request_id || 'hello',
        status: 'ok',
        findings: []
      }));
      return;
    }

    const findings = analyze(message.params?.content || '');
    console.log(JSON.stringify({
      protocol_version: PROTOCOL_VERSION,
      request_id: message.request_id || 'analyze',
      status: 'ok',
      findings
    }));
  } catch (_) {}
});
