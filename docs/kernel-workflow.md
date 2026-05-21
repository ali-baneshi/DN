# Kernel Workflow

## Example workflow

1. Build the tools:

```bash
cargo build --workspace
make -C workers/c
```

2. Scan a kernel tree:

```bash
dn-cli scan ~/src/linux --profile kernel-c --json
```

3. Review a finding such as `return-without-unlock` in `drivers/` or `mm/`.
4. Patch the file, rerun the scan, then validate with your usual kernel build or subsystem tests.
5. Generate and send the patch:

```bash
git format-patch -1
git send-email 0001-*.patch
```

## Walkthrough

```bash
dn-cli scan ~/src/linux/ipc --profile kernel-c --json --content
```

This catches kernel-style issues, risky `BUG_ON`, missing `KERN_` levels, cleanup mistakes around `goto out`, lock leaks on `return`, RCU dereference hazards, and sleeping calls in atomic sections before posting the patch.
