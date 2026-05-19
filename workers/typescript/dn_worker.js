#!/usr/bin/env node
const fs = require('fs');
const readline = require('readline');

const rl = readline.createInterface({ input: process.stdin, terminal: false });
let buffer = '';
rl.on('line', (line) => {
  buffer += line;
  try {
    const message = JSON.parse(buffer);
    buffer = '';
    if (message.method === 'hello') {
      console.log(JSON.stringify({ protocol_version: '0.1.0', worker_name: 'dn-typescript-worker', capabilities: ['typescript', 'javascript'] }));
      return;
    }
    const findings = [];
    if ((message.content || '').includes('eval(')) {
      findings.push({ rule: 'ts-eval-usage', severity: 'high', message: 'eval() usage detected', category: 'security', line: 1 });
    }
    console.log(JSON.stringify({ protocol_version: '0.1.0', findings }));
  } catch (_) {}
});
