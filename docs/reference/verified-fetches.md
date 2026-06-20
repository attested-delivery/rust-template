---
diataxis_type: reference
---
# Verified Fetches Reference

Precise rules for acquiring any external package, binary, or tool in a workflow.
Every fetch step must **pin an exact version**, **verify integrity**, and **fail
closed on mismatch**. For the rationale see the
[explanation](../explanation/verified-fetches.md); for step-by-step instructions
see the [how-to](../how-to/add-a-verified-tool.md).

## Preference order

Stop at the first option that applies.

| Order | Option | Integrity boundary |
|---|---|---|
| 1 | Use a tool **preinstalled on the runner** | The runner image (GitHub-maintained) |
| 2 | Use a **SHA-pinned action** | The full 40-char commit SHA (`pin-check` enforced) |
| 3 | **Download → verify → fail closed** | The integrity ladder below (never below a checksum) |
| 4 | **Package-manager install**, pinned | Lockfile / registry checksum database |

## Integrity ladder

When option 3 applies, use the **strongest mechanism the artifact actually
supports**. The floor is a pinned-digest checksum; nothing weaker is acceptable.

| Rank | Mechanism | Command | Notes |
|---|---|---|---|
| 1 (strongest) | Sigstore attestation | `gh attestation verify <file> --owner <o> --signer-workflow <wf>` | Keyless, transparency-logged |
| 2 | Detached signature | `cosign verify-blob` / `gpg --verify` / `minisign -V` | Requires a trusted public key/identity |
| 3 (floor) | Pinned-digest checksum | `echo "<sha256>  <file>" \| sha256sum -c -` | Digest from the publisher's **published** checksums file; leave a `# TODO` to upgrade |

Rules for every option-3 fetch:

- Run under `set -euo pipefail` so a failed check aborts the job.
- Resolve the digest/SHA **at use time** from the publisher's release, never
  from memory.
- Download to a file and verify **before** executing the artifact.
- Never pipe-to-shell (`curl … | sh`, `… | bash`, `… | tar`).

## Package-manager integrity

| Ecosystem | Required form | Integrity source |
|---|---|---|
| npm | `npm ci` | `package-lock.json` hashes |
| pnpm | `pnpm install --frozen-lockfile` | `pnpm-lock.yaml` hashes |
| yarn | `yarn --immutable` | `yarn.lock` hashes |
| Go | `go install <pkg>@<version>` | Go checksum database (`sum.golang.org`) |
| Rust | `cargo install --locked --version <X> <crate>` | crates.io index checksums + `Cargo.lock` |
| Node package managers | `corepack enable` | Signed package-manager keys |

Forbidden: unpinned installs (`cargo install foo`, `go install pkg@latest`) and
failure-swallowing suffixes (`… || true`).

## Preinstalled on `ubuntu-latest`

Common tools already present on GitHub-hosted Ubuntu runners — use directly, add
no install step. (Authoritative, version-exact list:
[actions/runner-images](https://github.com/actions/runner-images) Ubuntu
readme; verify with `which <tool>` in a scratch step.)

| Category | Tools |
|---|---|
| Shell / archive | `bash`, `curl`, `wget`, `tar`, `gzip`, `unzip`, `xz` |
| GitHub / VCS | `gh`, `git`, `git-lfs` |
| JSON / text | `jq`, `sed`, `awk`, `grep` |
| Languages | `node`, `npm`, `python3`, `pip`, `go`, `rustc`, `cargo`, `java` |
| Containers | `docker`, `docker compose`, `buildx`, `podman`, `skopeo` |
| Cloud CLIs | `aws`, `az`, `gcloud` |
| Build | `make`, `cmake`, `gcc`, `clang`, `pkg-config` |

Tools commonly **absent** (require option 2 or 3): `actionlint`, `cosign`,
`syft`, `grype`, `trivy`, `cargo-criterion`, `cargo-deny`, `cargo-llvm-cov`.

## How this is enforced

| Control | Where | Effect |
|---|---|---|
| `pin-check` | central `attested-delivery/.github` reusable | Fails any non-SHA `uses:` (required status check) |
| `actionlint` | `reusable-actionlint.yml` thin caller | Workflow-syntax lint via a verified, pinned fetch |
| `set -euo pipefail` | every fetch step | A failed checksum/verify aborts the job (fail-closed) |
| Allow-list | org Actions policy | Third-party actions blocked until owner-added |
