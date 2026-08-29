# Developer Certificate of Origin

Any contribution that does get accepted — see [CONTRIBUTING.md](CONTRIBUTING.md),
this is rare by design — must be signed off under the Developer Certificate of
Origin, version 1.1.

The canonical text lives at <https://developercertificate.org/>. Read it before
signing off; it is one page.

In short, signing off asserts that you wrote the contribution or otherwise have
the right to submit it under this project's licence, that you understand the
contribution and your sign-off are public and permanent, and that you are
willing to have that record kept indefinitely.

## How to sign off

Add a `Signed-off-by` trailer to each commit, using your real name and an
address you can be reached at:

```
git commit -s -m "fix: keep the period on unlisted abbreviations"
```

which appends:

```
Signed-off-by: Jane Doe <jane@example.com>
```

Configure it once:

```
git config user.name "Jane Doe"
git config user.email "jane@example.com"
```

Commits without a sign-off will not be merged. There is no CLA — the DCO is the
whole requirement.

## Why the DCO and not a CLA

A CLA asks contributors to assign or license rights to a legal entity. There is
no entity here. The DCO records that a contributor had the right to send what
they sent, which is the actual thing this project needs to be able to prove —
especially for linguistic reference data, where the provenance question is
"where did this word list come from" and the honest answer must be documented.