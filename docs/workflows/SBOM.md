---
id: reference-sbom-workflow
type: semantic
created: '2026-07-02T00:00:00Z'
modified: '2026-07-02T00:00:00Z'
namespace: reference/workflows
title: release.yml sbom job — GitHub Actions workflow reference
tags:
  - reference
  - ci
  - workflow
  - sbom
  - release
temporal:
  '@type': TemporalMetadata
  validFrom: '2026-07-02T00:00:00Z'
  recordedAt: '2026-07-02T00:00:00Z'
  ttl: P1Y
provenance:
  '@type': Provenance
  sourceType: system_generated
  trustLevel: verified
  wasDerivedFrom:
    '@id': https://github.com/modeled-information-format/mif-rs/blob/main/.github/workflows/release.yml
    '@type': prov:Entity
citations:
  - '@type': Citation
    citationType: tool
    citationRole: source
    title: anchore/sbom-action
    url: https://github.com/anchore/sbom-action
  - '@type': Citation
    citationType: specification
    citationRole: source
    title: CycloneDX Specification
    url: https://cyclonedx.org/specification/overview/
  - '@type': Citation
    citationType: specification
    citationRole: methodology
    title: Diátaxis — Reference
    url: https://diataxis.fr/reference/
    accessed: '2026-07-02'
ontology:
  '@type': OntologyReference
  id: mif-docs
  version: 1.0.0
  uri: https://mif-spec.dev/ontologies/mif-docs
entity:
  name: release.yml sbom job
  entity_type: reference-document
---

# release.yml — sbom job

The `sbom` job in `.github/workflows/release.yml` generates a CycloneDX JSON
Software Bill of Materials over the release's built binaries and attests it,
binding the SBOM to every published binary artifact.

## Synopsis

```yaml
on:
  push:
    tags: ["v*.*.*"]
  workflow_dispatch:
```

## Triggers

| Event | Condition |
| --- | --- |
| `push` (tag) | Tag matches `v*.*.*` |
| `workflow_dispatch` | Manual dry-run from any branch (produces no persistent attestation) |

## Job

| Job ID | Name | Runs on | `needs` |
| --- | --- | --- | --- |
| `sbom` | SBOM (generate + attest) | `ubuntu-latest` | `[meta, build]` |

## Permissions

| Scope | Level |
| --- | --- |
| `contents` | `read` |
| `id-token` | `write` |
| `attestations` | `write` |

## Steps

| Step | Action | Pin | Effect |
| --- | --- | --- | --- |
| Checkout repository | `actions/checkout` | `9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` (v7.0.0) | Fetch source |
| Download all binaries | `actions/download-artifact` | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` (v8.0.1) | Pattern `${bin}-${version}-*-*` into `dist/` |
| Generate CycloneDX SBOM | `anchore/sbom-action` | `e22c389904149dbc22b58101806040fa8d37a610` (v0.24.0) | `path: .`, `format: cyclonedx-json`, output `${bin}-${version}-sbom.cdx.json` |
| Attest SBOM | `actions/attest-sbom` | `c604332985a26aa8cf1bdc465b92731239ec6b9e` (v4.1.0) | Tag-only (`if: startsWith(github.ref, 'refs/tags/')`); `subject-path: dist/*`, binds every binary to the SBOM |
| Upload SBOM artifact | `actions/upload-artifact` | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` (v7.0.1) | Name `${bin}-${version}-sbom` |

## Binary name / version resolution

Resolved once in the upstream `meta` job (`needs: [meta, build]`), from
`cargo metadata --no-deps --locked --format-version 1`:

| Output | Source | Logic |
| --- | --- | --- |
| `bin` | `.packages[0].targets[]` | First target of the first package (by `cargo metadata` order) whose `kind` includes `"bin"` |
| `version` | `GITHUB_REF` or `.packages[0].version` | `refs/tags/v*` → strip the `v` prefix; otherwise (`workflow_dispatch`) `<Cargo.toml version>-dev` |

`meta` has no `[[bin]]` target found: if `.packages[0]` does not declare a
`[[bin]]` target, `meta` fails immediately with
`::error::no [[bin]] target found in Cargo.toml` and `exit 1`, before `build`
or `sbom` can run.

**Current repository state**: `cargo metadata`'s package order in this
workspace is `mif-core, mif-schema, mif-ontology, mif-cli, mif-mcp`.
`.packages[0]` is `mif-core`, a library crate with no `[[bin]]` target
(`mif-cli` and `mif-mcp` are the only crates with `[[bin]]` targets). `meta`
therefore fails on this workspace as currently configured, and `sbom` does
not run. See the workspace `CLAUDE.md` for the planned fix (binary resolution
driven off `.packages[] | select(...)` across all packages, not
`.packages[0]`).

## SBOM contents (shape)

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "metadata": {
    "component": { "type": "application", "name": "mif-cli", "version": "0.1.0" }
  },
  "components": [
    {
      "type": "library",
      "name": "serde",
      "version": "1.0.228",
      "licenses": [{ "license": { "id": "MIT" } }],
      "purl": "pkg:cargo/serde@1.0.228"
    }
  ]
}
```

## Artifacts

| Name | Path | Notes |
| --- | --- | --- |
| `${bin}-${version}-sbom` | `${bin}-${version}-sbom.cdx.json` | Also attached to the GitHub Release by the downstream `release` job |

## Attestation

| Field | Value |
| --- | --- |
| Predicate type | `https://cyclonedx.org/bom` |
| Subject | Each file under `dist/*` (the platform binaries) |
| Verify | `gh attestation verify <binary> --repo <owner>/mif-rs --predicate-type https://cyclonedx.org/bom` |

## Examples

Generate an equivalent CycloneDX SBOM locally with Syft (the engine behind
`anchore/sbom-action`):

```bash
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin
syft dir:. -o cyclonedx-json > sbom.cdx.json
```
