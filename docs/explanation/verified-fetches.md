---
diataxis_type: explanation
---
# Why Verify Every External Fetch

This document explains *why* every workflow step that downloads or installs a
package, binary, or tool must pin an exact version, verify integrity, and fail
closed on mismatch. For the precise rules and the preference ladder see the
[verified-fetches reference](../reference/verified-fetches.md); for the steps to
add a new tool safely see the [how-to](../how-to/add-a-verified-tool.md).

## The threat: a build is only as trustworthy as what enters it

The organization's central promise is that *the thing you verified is the thing
that runs* — signed, SLSA-attested, refused at the door if it cannot be proven.
That promise is worth nothing if the build itself was assembled from untrusted
bytes. Every step that pulls something external into a job — a release tarball, a
`go install`, a `curl … | sh` bootstrap, an unpinned `cargo install` — is a
supply-chain entry point. If any one of them can be silently swapped, an attacker
does not need to break your signing: they poison the input *before* you sign it,
and your attestation faithfully certifies the compromised result.

This is not hypothetical. The March 2026 `trivy-action` compromise
(CVE-2026-33634) — the reason this org pins every `uses:` to a full commit SHA —
was exactly this class of attack: a mutable reference repointed at malicious
code that ran inside other people's builds. A tarball fetched by version *tag*,
an install script piped straight into a shell, or a tool resolved to "latest"
carries the same mutability risk as an unpinned action.

## Why "pin the version" is not enough on its own

Pinning a version answers *which* artifact you wanted; it does not prove you
*got* that artifact. A registry can be compromised, a mirror can be poisoned, a
download can be intercepted. Integrity verification — a checksum, a signature, or
an attestation — closes the gap between "I asked for v1.7.7" and "the bytes I am
about to execute are genuinely v1.7.7." The two requirements are independent and
both are mandatory:

- **Pin the version** → eliminates "latest" / floating drift; makes the build
  reproducible and the intended artifact explicit.
- **Verify integrity** → eliminates substitution; proves the pinned bytes are
  the real bytes.

## Why fail-closed, never fail-open

A verification step that logs a warning and continues is theater. The entire
value of the check is that a mismatch *stops the build*. Under `set -euo
pipefail`, a failed `sha256sum -c` aborts the job; an `… || true` appended to an
install, or a verification whose exit code is ignored, converts a hard security
boundary into a decorative one. The same logic forbids `curl … | sh`: piping
bytes straight into an interpreter executes them *before* any check can run —
there is no point at which a mismatch could halt anything.

## Why a ladder instead of one rule

Not every publisher offers the same evidence. Some ship Sigstore-signed
provenance you can check with `gh attestation verify`; some ship a detached
signature; many ship only a checksums file. Rather than block adoption on the
strongest mechanism, the practice uses the *strongest mechanism each artifact
actually supports*, with a hard floor: a pinned-digest `sha256sum -c`. Where only
a checksum is available, a `# TODO` records the intent to upgrade once the
publisher ships signed provenance. This keeps the floor non-negotiable while
letting the ceiling rise as the ecosystem improves.

## Why "prefer the runner" comes first

The cheapest fetch to secure is the one you never make. GitHub-hosted runners
ship a large, known, GitHub-maintained toolset — `gh`, `jq`, `git`, `curl`,
`tar`, `node`, `go`, `python`, `docker`, and more. The runner image *is* a trust
root: it is built, versioned, and published by the same platform that runs the
job. Using a preinstalled tool directly removes an entire download-and-verify
step and shrinks the attack surface to nothing. Only when a tool is genuinely
absent does the ladder begin.

## Why this is the adopter's job in *every* workflow

The release pipeline is the obvious place to care about supply-chain integrity,
but it is not the only entry point. A linter fetched in CI, a benchmark tool in a
nightly job, a doc generator in a Pages deploy — each runs with the repository's
credentials and can taint anything downstream. The practice therefore applies
uniformly: *every* workflow, not just `release.yml`. Centralizing the verified
fetch in a reusable workflow (so callers consume it as a thin caller rather than
re-implementing `curl`-and-checksum logic) is how the org keeps that uniformity
from rotting into copy-pasted drift.
