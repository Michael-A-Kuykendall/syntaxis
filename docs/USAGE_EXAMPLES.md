# Usage Examples

These examples are the functional-review script for Syntaxis. They show what
the current product does, what it returns, and where its boundary is.

## 1. Embed the Rust API

Run the checked-in consumer example:

```sh
cargo run --example api_consumer --offline
```

It exercises:

- Canonical JSON analysis and a grammar diagnostic.
- Repeatable SHA-256 digest output.
- CoNLL-U export from analyzed text.
- Strict CoNLL-U import of the checked-in gold fixture.
- Empty, oversized, and Unicode input handling.

The API is I/O-free. The caller owns files, networking, queues, and output
storage. Use `syntaxis::api` when a consumer wants strings and deterministic
errors rather than the internal `Analysis` graph.

## 2. Use the CLI in a pipeline

The binary writes canonical JSON to stdout, so it can be composed with normal
pipeline tooling:

```sh
cargo run --offline -- "The cat are sleeping." > analysis.json
cargo run --offline -- --digest "The cat are sleeping."
cargo run --offline -- --validate "The cat are sleeping."
cargo run --offline -- --conllu-out "The cat are sleeping."
cargo run --offline -- --evaluation
```

The evaluation command is a construction-focused 500-case gate. It is not a
natural-language accuracy claim.

## 3. Run it in a browser

Build the optional package:

```sh
wasm-pack build --target web --no-default-features --features wasm
```

Then a browser application can use the generated module:

```js
import init, {
  analyze_conllu,
  analyze_json,
  digest,
  import_conllu_json,
} from "./pkg/syntaxis.js";

await init();
const text = "The cat are sleeping.";
const analysis = JSON.parse(analyze_json(text));
const stableDigest = digest(text);
const conllu = analyze_conllu(text);
const imported = import_conllu_json(conllu);
```

The browser API returns strings or deterministic errors. It does not read
files, access a browser filesystem, contact a service, or provide a UI.
`review.html` is a repository-only harness for this flow.

## 4. Preserve and exchange CoNLL-U

For a hand-annotated or upstream parse, import strict CoNLL-U and inspect the
returned canonical JSON:

```sh
cargo run --offline -- \
  --conllu-in fixtures/challenge_agreement.conllu
```

Import validates rows, spans, `# text`, and supported mappings. It preserves
the supplied annotations as imported evidence; it does not infer new grammar
diagnostics from them. The bundled fixture round-trips byte-for-byte through
the native import/export path.

Use `analyze_conllu` when the source is plain English and Syntaxis should
produce the bounded structure. Use CoNLL-U import when another system already
supplied the annotation.

## 5. Inspect uncertainty and retraction

Consumers that need the graph-level contract can use the public model modules:

```rust
use syntaxis::english_rules::{pipeline::analyze, rulepack::RulePack};
use syntaxis::parser_core::support::SourceRef;

let pack = RulePack::builtin()?;
let mut analysis = analyze("The cat are sleeping.", &pack);
let subject_arc = analysis
    .arcs
    .values()
    .find(|arc| arc.relation.as_str() == "nsubj")
    .ok_or("missing subject arc")?
    .id;
let report = analysis.retract(&SourceRef::Arc(subject_arc));
assert!(!report.removed.is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Derived facts carry rule and source supports. Retraction removes dependent
facts rather than leaving stale diagnostics for the caller to clean up.

## Boundary cases

Use Syntaxis when the consumer needs reproducible, inspectable structure for
the declared construction set. Do not use the current release as a complete
English grammar, spelling corrector, semantic parser, or broad proofreading
replacement. Unknown words, unsupported relations, and ambiguous structures
remain explicit instead of being guessed.
