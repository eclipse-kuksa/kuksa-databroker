<!--
Copyright (c) 2026 Contributors to the Eclipse Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Actuation subscription performance review

Review date: 21 October 2026. Work references: [issue #233](https://github.com/eclipse-kuksa/kuksa-databroker/issues/233), [PR #234](https://github.com/eclipse-kuksa/kuksa-databroker/pull/234), and the preceding [merged PR #232](https://github.com/eclipse-kuksa/kuksa-databroker/pull/232).

## Scope and contract

The candidate stores immutable actuation subscriptions behind `Arc`. A command snapshots the subscription references under a read lock, releases that lock, locates its provider and then awaits provider I/O. This avoids deep copies of each subscription's signal IDs and permissions. Lookup remains linear; the patch does not introduce the optional index discussed in #233.

The gRPC schema, routing, caller validation, provider-expiry checks, provider availability checks and error propagation stay unchanged. Repeated changes to the same signal retain their input order. Cross-signal batch ordering and existing partial-delivery behavior are not changed or given new guarantees.

| Boundary | Evidence |
| --- | --- |
| Correct single and batch routing, including repeated IDs | `single_and_batch_requests_reach_the_matching_providers` |
| Denied, missing or unavailable provider | `denied_and_unavailable_requests_are_not_delivered` |
| Provider registration while a batch is pending | `pending_batch_does_not_block_provider_registration` |
| Expired provider before and after housekeeping cleanup | `test_expired_actuation_provider_is_not_called` |
| Clearing subscriptions during single and batch provider I/O | `clearing_subscriptions_preserves_in_flight_commands_only` |

The last two tests were added in this review. They are compatibility controls, not claims that the previous implementation failed functionally. The expiry test constructs the post-expiry state directly, avoiding sleeps. The lifecycle test checks that clearing the registry does not wait for provider I/O, that the in-flight call finishes, and that later calls cannot use the removed subscription.

## Revisions and reproduction

- Upstream `main` and candidate base: `7a210d01fd00a8035cf68c05336bd0860777e892`, verified by a fresh upstream fetch.
- Published candidate: `7860f9e050beff35513dc869f5d87bf9007aeaad`.
- Follow-up: only the two additional tests and this review record; production behavior is unchanged from the published candidate.
- Environment: Linux x86_64, Rust/Cargo 1.99.1, CMake 4.4.4, locked Cargo dependencies and proto submodule `28bf76ae57085dad425eedd95bc7bd442e1314a7`.
- Debug/test symbols are disabled locally to limit build storage (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`); build parallelism is 4.

Run from the repository root after `git submodule update --init --recursive`:

```sh
cargo test --locked -p databroker --lib --test actuation_routing --test read_write_values
cargo fmt -- --check
cargo check --locked --all
cargo clippy --locked --all-targets -- -W warnings -D warnings
cargo clippy --locked --features viss --all-targets -- -W warnings -D warnings
pre-commit run --files databroker/src/broker.rs databroker/tests/actuation_routing.rs databroker/examples/actuation_benchmark.rs doc/actuation-performance-review.md
```

The restored candidate plus both new tests passed locally:

- Library: **195 passed, 1 existing ignored** (`glob::tests_glob_matching::test_matches_combination_of_multiple_wildcard_and_single_wildcard`).
- Actuation routing integration tests: **4 passed**.
- Cucumber: **53 scenarios / 183 steps passed**.
- Two deliberate faults were rejected by the new tests: bypassing the provider-expiry checks fails with the unexpected-provider panic; retaining the subscriptions read lock across provider I/O fails the bounded shutdown assertion. Each mutation was reverted, and the complete command above was rerun successfully.
- SHA-256 of the two-test diff against published head (before this documentation file): `385380232e4db6d5344055a81f053da3db8d8f6b7e936827472ed81c81368b19`.

Static checks passed: `cargo fmt`, locked workspace `cargo check`, default and VISS `cargo clippy --all-targets -- -W warnings -D warnings`, full configured pre-commit on the changed files, and `git diff --check`. Pre-commit's YAML check skipped because no YAML file changed. No new skips or lint suppressions were introduced.

## Performance evidence and limits

The published PR reports 31–53% lower in-process single-command overhead for 16 providers / 1,024 registered IDs. Those September measurements were not repeated for this test-only follow-up and are not new October results. Batch results were mixed and noisy. The unchanged benchmark is `databroker/examples/actuation_benchmark.rs`:

```sh
cargo run --locked --release -p databroker --example actuation_benchmark
```

For a before/after comparison, copy that same example onto the pinned base and use identical toolchains, build settings and workload arguments on the same host. The benchmark excludes registration and transport and uses `ALLOW_ALL`; it does not establish vehicle latency, concurrent throughput or a universal improvement.

## Review status and project requirements

The contribution guide requires formatting, Clippy, pre-commit and Eclipse contributor/sign-off requirements. The original candidate commit has a sign-off and its Eclipse ECA check is successful. Follow-up commits must also meet those requirements; the PR records their current check results.

At the pre-submission check on 21 October, the draft had no human review comments. The five upstream workflows report `action_required`, with no debug-workflow jobs listed. Upstream CI approval and execution remain external dependencies; local results must not be described as upstream CI success. Container, cross-architecture, all-feature coverage and vehicle validation are not established by these local checks.

| Contribution record | State |
| --- | --- |
| Preparation effort | Not measured; do not infer active work time from elapsed dates |
| First submission | 26 September 2026, 14:00:52 UTC; validation claims recorded in PR body |
| Avoidable external correction rounds | None visible in #234 as of this review |
| Design discussion | #233 asks for quantified benefit or simpler code; an index remains optional |
| Review wait | No human response visible between submission and 21 October review |
| Outcome | Compatibility-test follow-up for PR #234; acceptance and merge pending |
| Time to acceptance | Pending |
| Reusable lesson | Preserve short lock ownership and report measurements separately from transport or vehicle behavior |

Prepared with AI assistance for Timothy Anderson — Spotted Ridge Engineering / OC Labs, using GDN (Global Decision Network).

timothyanderson096@gmail.com · [Engineering evidence](https://github.com/timothyanderson096-ocdealcheck/gdn-engineering-benchmarks)
