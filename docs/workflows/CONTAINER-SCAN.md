---
id: reference-container-scan-workflow
type: semantic
created: '2026-07-02T00:00:00Z'
modified: '2026-07-02T00:00:00Z'
namespace: reference/workflows
title: Container vulnerability scanning (Trivy) — GitHub Actions workflow reference
tags:
  - reference
  - ci
  - workflow
  - trivy
  - container
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
    '@id': https://github.com/modeled-information-format/mif-rs/blob/main/.github/workflows/quality-gates.yml
    '@type': prov:Entity
citations:
  - '@type': Citation
    citationType: tool
    citationRole: source
    title: Trivy
    url: https://aquasecurity.github.io/trivy/
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
  name: Container vulnerability scanning (Trivy)
  entity_type: reference-document
---

# Container vulnerability scanning (Trivy)

Trivy scanning in this repo has no dedicated workflow file. It runs through
the central reusable `reusable-trivy.yml` (from
`modeled-information-format/.github`), called from two jobs in two different
caller workflows, plus one attestation job.

## Callers

| Caller workflow | Job ID | Job name | Scan target | Gated by |
| --- | --- | --- | --- | --- |
| `.github/workflows/quality-gates.yml` | `trivy` | (unnamed) | Filesystem (source tree: Dockerfile, manifests, licenses) | Always runs (see Triggers) |
| `.github/workflows/pipeline.yml` | `gate-image` | Gate — Trivy (image) | Built container image, by digest | `needs: [docker]`; `docker` itself gated on `publishable == 'true'` |
| `.github/workflows/pipeline.yml` | `attest-container-scan` | Attest — Container scan | Signs the `gate-image` verdict | `needs: [docker, gate-image]` |

## Triggers

The `quality-gates.yml` filesystem scan inherits that workflow's top-level triggers:

| Event | Condition |
| --- | --- |
| `push` | Branch `main` |
| `pull_request` | Target branch `main` |
| `schedule` | `0 6 * * 1` (Monday 06:00 UTC) |
| `workflow_dispatch` | Manual |

The `gate-image`/`attest-container-scan` jobs in `pipeline.yml` are event-driven
(not scheduled): they run when `github.event_name != 'pull_request'` and,
for `workflow_dispatch`, only when `inputs.stage` is `all` or `docker`. They
additionally require `needs.gate.outputs.publishable == 'true'`, which reads
`publish` from each workspace crate's `Cargo.toml`.

## Reusable workflow invocations

| Job | Reusable | Pin | `with:` |
| --- | --- | --- | --- |
| `trivy` (quality-gates.yml) | `reusable-trivy.yml` | `e50b004cbdcf2b3258d223b1f6a4d98ff7938abf` | `scan-iac: true` (no `image-ref`; filesystem mode) |
| `gate-image` (pipeline.yml) | `reusable-trivy.yml` | `e50b004cbdcf2b3258d223b1f6a4d98ff7938abf` | `image-ref: ghcr.io/${{ github.repository }}@${{ needs.docker.outputs.image-digest }}`, `scan-iac: false` |
| `attest-container-scan` (pipeline.yml) | `reusable-attest-scan.yml` | `e50b004cbdcf2b3258d223b1f6a4d98ff7938abf` | `subject-name: ghcr.io/${{ github.repository }}`, `subject-digest: ${{ needs.docker.outputs.image-digest }}`, `predicate-type: https://modeled-information-format.github.io/attestations/container-scan/v1` |

## Permissions

| Job | `contents` | `security-events` | `actions` | `packages` | `id-token` | `attestations` |
| --- | --- | --- | --- | --- | --- | --- |
| `trivy` | `read` | `write` | `read` | `read` | — | — |
| `gate-image` | `read` | `write` | `read` | `read` | — | — |
| `attest-container-scan` | `read` | — | — | — | `write` | `write` |

## What it scans for

| Category | Coverage |
| --- | --- |
| OS package vulnerabilities | CVEs in image base-layer packages |
| Application dependencies | Vulnerabilities in bundled application dependencies |
| Misconfigurations | Dockerfile/IaC misconfiguration rules (filesystem scan, `scan-iac: true`) |
| Secrets | Secrets embedded in image layers |

Severity levels: `CRITICAL`, `HIGH`, `MEDIUM`, `LOW`, `UNKNOWN`.

## Current repository state

`docker`, `gate-image`, and `attest-container-scan` are gated on
`needs.gate.outputs.publishable == 'true'`. As of this writing, `mif-core`,
`mif-schema`, and `mif-ontology` each declare `publish = false` in their
`Cargo.toml`, so `gate` resolves `publishable = false` and the image-scan and
attestation chain does not run. The filesystem scan (`trivy` job in
`quality-gates.yml`) is unaffected by this gate and runs unconditionally on
its own triggers.

## Outputs (SARIF)

Filesystem findings upload to **Security tab → Code scanning alerts**. Image
scan findings become the `gate-image.outputs.image-sarif-artifact` /
`image-sarif-filename` pair, consumed by `attest-container-scan` and signed as
a `container-scan/v1` attestation bound to the image digest — verifiable with
`gh attestation verify`.

```json
{
  "results": [
    {
      "ruleId": "CVE-2021-12345",
      "level": "error",
      "message": { "text": "openssl: buffer overflow vulnerability" },
      "locations": [
        { "physicalLocation": { "artifactLocation": { "uri": "Dockerfile" } } }
      ]
    }
  ]
}
```

## Examples

Reproduce the filesystem scan locally:

```bash
trivy fs --scanners vuln,misconfig,secret .
```

Reproduce the image scan locally, against a locally built image:

```bash
docker build -t mif-rs:local .
trivy image mif-rs:local
```
