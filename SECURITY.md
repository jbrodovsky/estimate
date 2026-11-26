# Security Policy

## Supported versions

**estimate** is pre-1.0. Only the latest release, and `main`, get security fixes.

| Version         | Supported |
| --------------- | --------- |
| latest / `main` | yes       |
| anything older  | no        |

## Reporting a vulnerability

**Please don't open a public issue for a security vulnerability.**

Report it privately through GitHub's
[private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability):
on the repository's **Security** tab, choose **Report a vulnerability**. This opens a private
advisory that only the maintainers can see.

Please include:

- the affected version or commit, and the enabled features (`std`, `alloc`, `libm`),
- the target (for example `x86_64-unknown-linux-gnu` or `thumbv7em-none-eabihf`),
- a minimal reproduction, and
- the impact as you understand it.

You should get an acknowledgement within a week. Once a fix is ready, we'll agree on the
disclosure timing with you and credit you in the advisory, unless you'd prefer not to be named.

## Scope

The library forbids `unsafe` code and, by policy, never panics in library code: it returns a
`FilterError` instead. A panic that untrusted input can reach is therefore a bug, and it may be
a denial-of-service issue for embedded users. If it can be triggered from outside, report it
privately as above. Otherwise, a normal bug report is fine.
