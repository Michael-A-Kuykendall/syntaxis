# Security policy

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Report privately through GitHub's [Security
Advisories](https://github.com/Michael-A-Kuykendall/syntaxis/security/advisories/new),
or by email to michaelallenkuykendall@gmail.com.

Include: what you found, how to reproduce it, the affected version, and the
impact as you understand it. A proof-of-concept input file is ideal.

### What to expect

| Stage | Target |
| --- | --- |
| Acknowledgement | 3 business days |
| Initial assessment | 10 business days |
| Fix or documented mitigation | depends on severity; you will be kept informed |

This is a single-maintainer project. These are response targets, not a
contractual SLA. If you have not heard back in a week, send a follow-up — mail
gets lost.

The release safety corpus exercises inputs up to 128 KiB, including long,
nested, malformed, and Unicode-heavy text. Larger callers should apply their
own input-size limit before invoking the engine; unbounded document processing
is not a supported contract.

## Supported versions

The most recent release only. There are no backported security fixes before
1.0.

## Scope

This is an offline text-analysis library. It opens no sockets, spawns no
processes, and reads no files it was not handed. The realistic threat model is
**hostile input text**, so these are in scope:

- Panics reachable from library APIs on any input, including malformed UTF-8
  handling, slicing on a non-character boundary, or an arithmetic overflow.
- Unbounded memory or CPU growth on a small input — quadratic blowup in
  segmentation or tokenization is a denial-of-service bug and will be treated
  as one.
- Any path where a span escapes that does not index the document it claims to.
- Stack exhaustion from deeply nested or pathological input.
- A checksum bypass that lets an unverified reference artifact load.

Also in scope, and taken seriously despite not being memory safety:

- **A non-determinism bug.** If you can produce two different outputs from the
  same input, version, and rule pack, that is a reportable defect against the
  project's central guarantee.
- **A provenance bug.** A derived fact whose `SupportSet` omits a source it
  materially depended on. This is the failure mode retraction is built to
  prevent: an unreported source is a fact that will not be retracted when it
  should be, and a consumer may act on a conclusion whose evidence is gone.

Out of scope: parse quality, tagging errors, and rules that miss constructions.
Those are ordinary bugs — file them as issues.

## Hardening notes

- `#![forbid(unsafe_code)]` across the workspace.
- Zero external dependencies, so there is no transitive supply chain.
- No network access, no filesystem access, no environment reads, no clock, no
  RNG anywhere in the analysis path.
- Reference artifacts are embedded at build time and SHA-256 verified at load;
  a mismatch is a hard failure, not a warning.
