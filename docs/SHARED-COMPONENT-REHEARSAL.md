# Prepare components and consuming apps together

English | [简体中文](SHARED-COMPONENT-REHEARSAL.zh-CN.md)

`shared_component_candidate` prepares a reviewable catalog transaction that
adds shared components and their consuming apps together. The ordinary
`publisher-entry` command resolves pins against an authenticated existing
catalog; this helper also resolves components newly added in the candidate.

Download real GitHub-attested releases into one directory. Include each
`<id>-<version>.component.json`, its matching `<id>-<version>.wasm`, and each
consumer's `*.bundle.pack.json`. Start from the current authentic
`catalog-v2.json`. No credentials or signing keys are used locally.

```sh
cargo run --locked -p octosense-app-hub --example shared_component_candidate -- \
  catalog-v2.json downloaded-releases new-candidate
```

The helper verifies the real admin base proof, verifies every component and
app publisher proof, resolves the exact pins, checks ownership continuity,
and runs the same final validation as `hub catalog-prepare`. It writes
`catalog.json`, all required artifact and index files, the canonical
`catalog-v2.payload.json`, and a receipt containing its SHA-256. It never
executes app or component code, authenticates its own output, or publishes.
Existing output directories are refused. Failures remove only a directory
newly created by that invocation.

Review the complete candidate and follow
[GitHub-admin publication](GITHUB-PUBLISHING.md). The protected workflow's
`dry_run: true` produces a real verified catalog envelope without changing
the public catalog. Use that envelope and the exact candidate artifacts for
isolated local installation; never substitute a fake proof or legacy test
catalog when claiming this end-to-end flow.

The example's three refusal/cleanup tests use the repository's genuine public
base proof. Success with newly attested components and consumers must be
recorded separately after a real publisher workflow run.
