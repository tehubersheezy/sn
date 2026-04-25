# Security Policy

`sn` handles ServiceNow credentials and connects to enterprise instances, so we take vulnerabilities seriously and appreciate responsible disclosure.

## Supported Versions

`sn` is in active 0.x development. Only the **latest minor release** receives security fixes; we do not backport to earlier 0.x lines. Once `sn` reaches 1.0, this policy will define a longer support window.

| Version | Supported |
| ------- | --------- |
| 0.3.x   | Yes       |
| < 0.3   | No        |

## Reporting a Vulnerability

**Please do not report security issues in public GitHub issues.**

Use one of these private channels:

1. **Preferred:** Open a [private security advisory](https://github.com/tehubersheezy/sn/security/advisories/new) on this repo. This gives both sides a private discussion thread and lets us coordinate a fix (and CVE assignment, if warranted) before public disclosure.
2. **Email:** `ibrahimsafah@gmail.com` with subject prefix `[sn-security]`.

When reporting, please include:

- A description of the issue and its impact
- Steps to reproduce, or a proof-of-concept
- Affected version(s)
- Any suggested mitigation

## What to Expect

- **Acknowledgement** within 7 days of report
- **Triage and status update** within 14 days, including a classification (none / low / medium / high / critical)
- **Fix or mitigation** for confirmed high/critical issues within 30 days where feasible; otherwise we will agree on a coordinated disclosure timeline with you
- **Credit** in the release notes and any CVE record, unless you request anonymity

## Scope

In scope:

- Vulnerabilities in this repository's source code
- Issues in the release pipeline that could allow malicious artifacts to be published
- Credential-handling issues (storage, transmission, leakage to logs/stdout)

Out of scope:

- ServiceNow platform vulnerabilities — report directly to ServiceNow
- Issues requiring physical access to the user's machine
- Issues in third-party dependencies (please report upstream; we will pull in fixes via Dependabot)
- Theoretical issues without demonstrable impact

## Defensive Posture

By default, `sn`:

- Verifies TLS certificates; `--insecure` is opt-in only and prints a warning
- Stores credentials separately from non-secret config (`credentials.toml`, chmod `0600` on Unix)
- Sends network traffic only to the configured ServiceNow instance and (optionally) the configured proxy — there is no telemetry or analytics
- Ships every release artifact built reproducibly via cargo-dist on GitHub-hosted runners

## Verifying Release Artifacts

Every release artifact is signed via [Sigstore](https://www.sigstore.dev/) using GitHub's [build provenance attestations](https://docs.github.com/en/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds). The signature is anchored in Sigstore's public Rekor transparency log and proves the artifact was built from this repo, by the release workflow, on a GitHub-hosted runner — without anyone holding a long-lived signing key.

To verify an artifact you downloaded:

```bash
# requires the GitHub CLI: https://cli.github.com/
gh attestation verify sn-x86_64-apple-darwin.tar.xz --owner tehubersheezy
```

A successful verification confirms:

- The artifact's SHA-256 matches what the build workflow produced
- The build ran in this repo (`tehubersheezy/sn`)
- The build was triggered by a tag push to `main`
- The signature is recorded in Sigstore's Rekor transparency log (auditable at <https://search.sigstore.dev/>)

Attestations are available for releases starting with `v0.3.4`.
