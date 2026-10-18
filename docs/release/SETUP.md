# Maintainer setup requiring decisions

These are reviewable proposals, not active release automation or completed sign-offs.
The readiness assessment lists the remaining owner confirmations.

## Main branch protection

[main-protection.json](main-protection.json) requires one approval and all eight CI
jobs plus the full-history secret scan, with no force pushes or deletion and no
administrator exemption. Code-owner approval is not separately required: the sole
current CODEOWNER cannot approve their own PR. Before activation, identify a second
reviewer with suitable repository access and check the exact CI context names.
The path-filtered supply-chain workflow is not a required status: it would leave
unrelated PRs waiting for a check that never runs. Dependency edits still run it.

Apply after agreement with:

```sh
gh api -X PUT repos/jasonduncan/semantic-operators-rs/branches/main/protection --input docs/release/main-protection.json
```

This changes the normal contribution path to reviewed PRs. No protection has been
silently waived or configured to permit administrative bypass.

## Trusted publishing

The [workflow proposal](trusted-publishing.yml.example) is inert here. After owner
sign-offs and a selected new release version:

1. Create the GitHub environment `crates-io` with a named maintainer reviewer. Limit
   deployment branches to the default branch for this workflow_dispatch design.
2. On crates.io, configure the exact trusted publisher: GitHub owner `jasonduncan`,
   repository `semantic-operators-rs`, workflow `release.yml`, environment `crates-io`.
3. Move the reviewed proposal to `.github/workflows/release.yml` through a PR. The
   owner must inspect the workflow on the default branch before dispatch.
4. Require passing candidate CI, secret scan, license audit and all launch sign-offs;
   create an immutable approved tag at that exact commit. Never move the alpha.1 tag.
5. Dispatch with the approved tag. The verification job runs a dry-run; the publish
   job requires the environment review and uses a temporary OIDC-derived token.
6. Verify registry checksum, source tag, docs.rs and a fresh consumer; prepare release
   notes from the actual shipped revision. Do not retry an uncertain upload blindly.

The template has been syntax-reviewed only; it has not obtained OIDC credentials or
published. A local Cargo dry-run does not prove trusted publishing authentication.
The repository currently has no Actions secrets or environments. No token has been
added. Account configuration and owner 2FA remain to be verified by the owner.

References: [crates.io trusted publishing](https://crates.io/docs/trusted-publishing),
[official auth action](https://github.com/rust-lang/crates-io-auth-action),
[GitHub branch protection](https://docs.github.com/en/rest/branches/branch-protection).
