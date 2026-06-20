---
diataxis_type: how-to
---
# How to Add a Tool to a Workflow Safely

This guide shows how to bring an external package, binary, or tool into a
GitHub Actions job without opening a supply-chain hole. Work top-down: stop at
the first option that applies. For the rationale see the
[verified-fetches explanation](../explanation/verified-fetches.md); for the full
ladder and the preinstalled-tool list see the
[reference](../reference/verified-fetches.md).

## 1. First, check whether the runner already ships it

1. Look up the tool in the runner image manifest (for `ubuntu-latest`, the
   [actions/runner-images](https://github.com/actions/runner-images) Ubuntu
   readme, or the [reference table](../reference/verified-fetches.md#preinstalled-on-ubuntu-latest)).
2. If it is preinstalled, **use it directly and add no install step**.

Verify:

```bash
# In a scratch step, confirm the tool resolves on the runner:
which jq && jq --version
```

If this prints a path and version, you are done — no fetch, nothing to verify.

## 2. Otherwise, use a SHA-pinned action

1. Find an action that installs the tool.
2. Resolve its release tag to a full 40-char commit SHA **at use time**:

   ```bash
   gh api repos/<owner>/<repo>/git/ref/tags/<tag> --jq .object.sha
   ```

3. Pin the `uses:` to that SHA with a trailing version comment:

   ```yaml
   - uses: anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610 # v0.24.0
   ```

4. If the action's publisher is not `actions/*`, `github/*`, or
   `attested-delivery/*`, it must be on the org allow-list **before** the
   workflow references it (an owner action), or the workflow startup-fails.

Verify:

```bash
actionlint .github/workflows/<file>.yml   # passes
# pin-check (central required status check) confirms the SHA pin on push/PR.
```

## 3. Otherwise, download → verify → fail closed

Use this only when no preinstalled tool and no pinned action exist.

1. Pin the exact version and resolve the strongest available integrity
   mechanism (see the [ladder](../reference/verified-fetches.md#integrity-ladder)).
2. Obtain the pinned digest from the release's **published** checksums file
   (e.g. `tool_<version>_checksums.txt`) — never from memory.
3. Download to a file, verify, then run, all under `set -euo pipefail`:

   ```bash
   set -euo pipefail
   VERSION="1.7.7"
   SHA256="023070a287cd8cccd71515fedc843f1985bf96c436b7effaecce67290e7e0757"
   curl -sSfL -o tool.tar.gz \
     "https://github.com/<owner>/<repo>/releases/download/v${VERSION}/tool_${VERSION}_linux_amd64.tar.gz"
   echo "${SHA256}  tool.tar.gz" | sha256sum -c -   # aborts the job on mismatch
   tar xzf tool.tar.gz -C /usr/local/bin tool
   tool --version
   ```

4. If only a checksum (not a signature/attestation) was available, leave a
   `# TODO` to upgrade when the publisher ships signed provenance.
5. **If more than one workflow needs this tool, do not copy the block** — lift
   it into a central reusable workflow and consume it as a thin caller (this is
   how `actionlint` is handled: the verified fetch lives once in
   `attested-delivery/.github/.github/workflows/reusable-actionlint.yml`).

Verify:

```bash
# The job fails closed: corrupt the SHA in a scratch branch and confirm the
# step exits non-zero (sha256sum: WARNING: 1 computed checksum did NOT match).
```

## 4. For package-manager installs, use lockfile / registry integrity

Pin the version and let the package manager's own integrity machinery fail
closed. Never unpinned, never failure-swallowing (`… || true`).

| Ecosystem | Use | Not |
|---|---|---|
| npm | `npm ci` | `npm install` (no lockfile enforcement) |
| pnpm | `pnpm install --frozen-lockfile` | `pnpm install` |
| yarn | `yarn --immutable` | `yarn` |
| Go | `go install pkg@v1.2.3` | `go install pkg@latest` |
| Rust | `cargo install --locked --version 1.1.0 <crate>` | `cargo install <crate>` |
| package managers | `corepack enable` (signed keys) | third-party setup action |

Verify:

```bash
# The lockfile / --locked / checksum DB is the integrity check; a tampered
# dependency makes the install exit non-zero.
cargo install --locked --version 1.1.0 cargo-criterion
```

## Never: pipe-to-shell

`curl … | sh`, `curl … | bash`, and `curl … | tar` execute unverified bytes
with no point at which a check could run. Always download to a file, verify,
then run.
