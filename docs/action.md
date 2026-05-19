# Official GitHub Action

`dn-kernel` ships with an official composite GitHub Action at `.github/actions/dn-kernel` for repositories that want a simple CI entrypoint.

## Inputs

- `profile`
- `path`
- `fail-on`
- `summary-only`
- `markdown`
- `hidden`

## Outputs

- `report-path`

## Example

```yaml
- uses: ./.github/actions/dn-kernel
  with:
    profile: pre-merge
    fail-on: medium
    summary-only: true
```

## Notes

- the action builds `dn-cli` from source inside the workflow
- JSON is the default report format for automation
- Markdown can be enabled for human review artifacts
