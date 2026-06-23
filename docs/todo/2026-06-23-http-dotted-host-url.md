# HTTP Dotted Host URL Fix

Date: 2026-06-23

## Problem

`grafana-util dashboard export --url https://hostx.0.2.120:3000 ...` fails before
the first Grafana request:

```text
Invalid URL for request path /api/org: invalid IPv4 address
```

The host is a DNS-shaped name, not an IPv4 literal. The Rust HTTP transport
should not surface this as an operator-facing `Invalid URL` failure.

## Scope

- Primary code path: `rust/src/grafana/http.rs`.
- Regression tests: `rust/src/grafana/http_rust_tests.rs`.
- Keep dashboard export behavior unchanged except that DNS-shaped hosts that
  look IPv4-like report an unknown-host style error instead of `Invalid URL`.
- Do not touch Python implementation, generated docs, command contracts, or
  dashboard export domain logic unless investigation proves the bug is there.

## Root Cause Hypothesis

`JsonHttpClient::build_url` uses `reqwest::Url`, whose parser follows WHATWG URL
host parsing. With a host like `hostx.0.2.120`, the parser treats the authority
as IPv4-like and rejects it before any DNS lookup can happen. Since reqwest
cannot construct the request URL for this shape, the CLI should classify the
edge as an unresolved host rather than a malformed operator URL.

## Execution Plan

1. Add a focused HTTP regression test for `https://hostx.0.2.120:3000` and
   `/api/org`.
2. Verify the new test fails with the current implementation.
3. Change URL error classification so this parser edge reports unknown host
   while malformed URLs still report `Invalid URL`.
4. Run the focused HTTP tests, then a relevant broader Rust test target.
5. Review follow-up: add coverage for numeric-suffix internal DNS names such as
   `https://grafana.prod.1:3000` and generalize the host classifier beyond the
   original four-label dotted shape.

## Acceptance Checks

- The focused regression test fails before the production-code change and passes
  after it.
- Existing invalid URL behavior still reports `Invalid URL for request path ...`.
- Existing query encoding remains unchanged.
- `cd rust && cargo test --quiet http_rust_tests` passes.
- Numeric-suffix DNS names that trigger reqwest's `invalid IPv4 address` parser
  edge report `Unknown host ...` instead of `Invalid URL ...`.

## Verification

- `cd rust && rtk cargo test --quiet http_rust_tests` - passed on 2026-06-23.
- `cargo test http_rust_tests -- --nocapture` failed before the review fix on
  2026-06-24 with `grafana.prod.1` still reporting `Invalid URL for request path
  /api/org: invalid IPv4 address`.
- `cargo test http_rust_tests -- --nocapture` passed after the review fix on
  2026-06-24: 10 passed, 0 failed.
- `make test` passed on 2026-06-24: Rust lib reported `1823 passed; 0 failed; 1
  ignored`, integration tests reported `7 passed` and `30 passed`, and doctests
  reported `0 failed`.

## Outcome

Review follow-up generalized the parser-edge classifier from the original
four-label shape to valid DNS-style hosts with numeric final labels, while
preserving real IPv4 URL parsing and malformed-label fallback behavior.
