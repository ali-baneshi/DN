# Distribution

`dn-kernel` is prepared for binary release distribution.

## Release workflow

`.github/workflows/release.yml` builds release archives for:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

## Homebrew

A starter formula is provided at `packaging/homebrew/dn-kernel.rb`.

Before publishing a formula externally, replace the placeholder SHA256 values with the real checksums from the tagged release artifacts.
