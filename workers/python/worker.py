#!/usr/bin/env python3
import json
import sys

PROTOCOL_VERSION = "1.0.0"

def main():
    msg = {
        "protocol_version": PROTOCOL_VERSION,
        "status": "ready",
        "worker": "python"
    }
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

if __name__ == "__main__":
    main()
