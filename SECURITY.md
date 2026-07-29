# Security

servicemaker is the internal tool that turns a Python or Node.js project into a
deployable ArangoDB platform service: it renders a Dockerfile and a Helm chart,
builds an image on top of one of the four base images in `baseimages/`, and can
push both. The security-relevant output of this repository is therefore not just
the CLI binary but the four base images published to Docker Hub
(`arangodb/py12base`, `arangodb/py12cugraph`, `arangodb/py12torch`,
`arangodb/node22base`) and the two reference service images
(`arangodb/test-service`, `arangodb/test-service-nodejs`).

## Reporting a vulnerability

Report privately, not in a public issue:

- GitHub private vulnerability reporting on this repository (Security tab, zero
  cost, no GitHub Advanced Security needed). Preferred, because it does not
  depend on an inbox.
- `security@arangodb.com` as a fallback.

Acknowledgement within 3 business days, a first assessment with a severity and a
remediation plan within 10 business days, and a fix or a documented, dated
disposition inside the window in the POA&M table below. We will credit reporters
who want it and coordinate disclosure timing; the default is public disclosure
once a fixed base image is published.

## Supported versions

| Artifact | Supported | Notes |
| --- | --- | --- |
| `servicemaker` CLI, latest release | Yes | statically linked x86_64 binary, built by `make release` |
| `arangodb/py12base:latest`, `py12cugraph`, `py12torch`, `node22base` | Yes | `:latest` only; no historical tags are patched |
| `arangodb/test-service`, `test-service-nodejs` | Yes | reference services, rebuilt with the base images |
| Anything older than the current `:latest` | No | rebuild from `main` |

The base images ship only a `:latest` tag, so a consumer pinning a digest gets no
security updates. Digest pinning plus a documented rebuild cadence is the open
item under Known gaps.

## Two-tier gate policy

Every scan in `.circleci/config.yml` sits in exactly one of two tiers. The split
is deliberate: a gate that fails on findings nobody can act on gets switched off,
and a report that nobody can read is not evidence.

**Blocking tier (fails the job, and once the ruleset exists, blocks the merge)**

| Gate | Scope | Band |
| --- | --- | --- |
| `dependency-cve-scan` | `Cargo.lock`, `arango-test-service/requirements.txt`, `arango-test-service-nodejs/package-lock.json`, asserted by name with `expect-targets` | fixable CRITICAL, HIGH |
| `misconfig-scan` | every Dockerfile and both service templates, plus the final-stage build-credential guard | CRITICAL, HIGH |
| `sast-scan` | first-party code, `p/default` and `p/rust` | ERROR |
| `sast-scan-diff` | findings not present on `main` | WARNING |
| `disposition-check` | waiver and disposition integrity | any undated, expired, over-window or unattributed acceptance |
| six image gates | the four base images and the two reference services | fixable CRITICAL, HIGH (`scanners: vuln`), plus an end-of-life base OS |

**Report tier (annotates, never fails)**

- The full severity band (`CRITICAL` through `UNKNOWN`, unfixed included) is
  written to the JSON, table, JUnit and SBOM of every blocking scan through
  `report-severity`, so there is always evidence of what the gate did not count.
- `secret-scan` over the checkout, report-only for one enumeration cycle (see the
  flip criterion in `.circleci/trivy-secret.yaml`).
- A secret pass over each image filesystem and image config, report-only for the
  same reason and with the same flip criterion. Trivy's default scanner set for
  an image includes secret detection, so a `scanners: vuln` gate plus a separate
  report-only secret pass is what keeps a third-party fixture from failing a
  vulnerability gate. One such fixture is known and allow-ruled: tornado's test
  key, which ships inside `py12cugraph` because the uv archive cache is left in
  the image.
- `dependency-cve-scan-nightly`: full band, unfixed included, `--include-dev-deps`.
- `misconfig-scan-nightly`: widened to MEDIUM.
- KEV and EPSS correlation on every image gate: CISA BOD 26-04 moved the federal
  baseline off flat CVSS onto exploitation signals, and Trivy reports neither, so
  a KEV-listed MEDIUM would otherwise sit below every band here. It annotates and
  never fails, because an exploitation signal on an unfixable base-OS package is
  triage input, not a reason to block a build nobody can unblock.
- `required-checks-drift`, warn-only by construction.

UNKNOWN is never in a blocking band: it is a data-source property (some
vulnerability databases assign no CVSS at all), so gating on it would fail a
build on the absence of a score rather than on a risk.

## Remediation windows (POA&M)

Windows run from `detected`, and they are enforced, not aspirational:
`waiver-windows` in `.circleci/config.yml` and `poam-windows` in
`disposition-check` both carry this table, and a waiver whose expiry falls
outside its severity's window is rejected before the scan runs.

| Severity | Window | Notes |
| --- | --- | --- |
| CRITICAL | 30 days | |
| HIGH | 30 days | |
| MEDIUM | 90 days | |
| LOW | 180 days | |
| Design acceptance | 366 days | architectural, re-reviewed annually, not a schedule |
| On the CISA KEV catalogue | 90 days, capped, and never later than the CISA due date | BOD 26-04 |

A waiver cannot cure a breach: a finding already past its window has no valid
expiry date left, so the entry is rejected and the breach stays visible.

Open acceptances live in `.circleci/security-waivers.yaml`. There is one today:
`CVE-2026-14257` (brace-expansion, HIGH, DoS) bundled inside npm's own dependency
tree in the `node:22` layer, with no in-range fix and a dated exit criterion.

## Required status checks

`.github/required-checks.txt` is the authoritative list. **It is not enforced
yet**: this repository has no ruleset with required status checks, only the two
enterprise-sourced `stable-branch-no-force-push` and `stable-branch-no-delete`
rules, and `main` has no branch protection. Until an owner creates that ruleset,
every gate blocks pipeline progression and none of them blocks a merge.
`required-checks-drift` reports the gap on every run.

Context strings are unstable by nature: CircleCI appends an instance suffix when
a job name appears more than once in a pipeline, and the scheme is not
predictable. Every scan instance therefore carries an explicit `name:`, and the
strings are read off a live pipeline (`gh pr checks`) rather than derived.

## Artifact signing, attestation and SLSA level

**Not implemented. SLSA Build L0 for the published images.** The gates prove a
freshly built image was clean before `make push`, and each scan stores a CycloneDX
SBOM as a job artifact, but nothing signs the pushed digest, so a consumer of
`arangodb/py12base:latest` has no way to verify that what they pulled is what CI
built. Wiring keyless cosign over the pushed digests needs two things this
repository cannot decide: the org-level Rekor publicity decision, and a change to
`baseimages/push.sh` so each `docker push` digest is captured (the tag is not a
usable signing subject because it is re-pushed on every rebuild). Both are on the
owner list rather than shipped half-done here.

## Scan-coverage boundaries, stated honestly

- **The chart template is not scanned.** `charts/` carries
  `name: {SERVICE_NAME}` / `version: {VERSION}` placeholders, so helm cannot load
  it and Trivy's helm scanner reads zero files from it. `helm lint --strict`
  therefore runs where the chart is real: inside the integration test, over the
  chart servicemaker renders. Hardening the rendered chart (seccompProfile,
  readOnlyRootFilesystem, dropped capabilities) is template work in
  `charts/templates/` and is on the owner list; a NetworkPolicy is deliberately
  not proposed, because a chart installed by the platform operator may only
  contain resource kinds the `arango-operator` ServiceAccount can read.
- **`Dockerfile.nodejs.template` needed a file pattern.** Trivy's dockerfile
  analyzer does not recognise that filename (measured on v0.72.0), so the Node.js
  service template was silently unscanned until `file-patterns` was set.
- **The uv archive cache ships inside the images.** `prepareuv.sh` leaves
  `/home/user/.cache/uv/archive-v0/` in the built image, which is why a
  third-party test fixture key is present in `py12cugraph` at all. Dropping the
  cache would shrink the images and remove the fixture; it is a change to
  `prepareuv.sh` and is on the owner list.
- **The published Docker Hub tags lag the gates.** The image jobs on a PR scan
  images built from that PR's Dockerfiles. What is on Docker Hub only changes
  when someone runs the `rebuild_base_images` pipeline, so a finding cleared here
  is not cleared for consumers until that rebuild.
- **The nightly cadence lives outside the repository.** The "Daily" scheduled
  pipeline (which sets `security_scan: true`) is CircleCI UI state, so deleting it
  silently removes the nightly. Since this change the image scans also run on
  every pipeline, so deletion would cost the daily drift cadence rather than all
  coverage. Migrating the schedule into a committed cron trigger is on the owner
  list.
- **`servicemaker` is a statically linked Rust binary**, so a consumer scanning
  the released binary sees nothing; `Cargo.lock` in this repository is the
  authoritative inventory.

## Known gaps

| Gap | Status | Cost of the paid alternative |
| --- | --- | --- |
| No required status checks on `main` | owner action; the committed list and the drift job are ready | free |
| No artifact signing or attestation | needs the Rekor publicity decision and a `push.sh` digest capture | free (cosign keyless, CircleCI OIDC to Fulcio) |
| Evidence not archived outside CircleCI | `publish-evidence` is wired and inert; no S3 bucket with Object Lock exists | S3 storage only |
| No continuous SBOM re-analysis | `dtrack-upload` is wired and inert; no Dependency-Track host exists | Dependency-Track is free and self-hosted; hosting cost only |
| SAST is Semgrep CE only | intentional | Semgrep AppSec Platform is roughly 40 USD per contributor per month, and the CE rule packs are the same rules |
| SARIF is not ingested by GitHub code scanning | intentional | GitHub Code Security is roughly 30 USD per active committer per month |
| Base images ship `:latest` only | no digest pinning or rebuild cadence for consumers | free |
| Rendered service chart is unhardened | template work, owner list | free |
| No `.pre-commit-config.yaml`, so no gitleaks pre-commit hook | no local hook adoption to build on; the repository-side secret scan is the control | free |

Reviewed with the monthly disposition review. Posture per control:
`.security/baseline.yml`.
