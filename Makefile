# Convenience targets. Everything here is plain cargo underneath; nothing in
# this file is required to build the project.

.PHONY: help build test test-quick lint fmt fmt-check audit demo checksums gate clean

help:
	@echo "build        release build"
	@echo "test         full test suite"
	@echo "test-quick   library tests only"
	@echo "lint         clippy, warnings as errors"
	@echo "fmt          format in place"
	@echo "fmt-check    verify formatting"
	@echo "audit        cargo-deny licence and ban check"
	@echo "demo         run the live tokenize/import/retract demo"
	@echo "checksums    print artifact checksums for the rule-pack manifest"
	@echo "gate         the automated subset of RELEASE_GATES_CHECKLIST.md"

build:
	cargo build --offline --release

test:
	cargo test --offline

test-quick:
	cargo test --offline --lib

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

audit:
	cargo deny check

demo:
	cargo run --offline --example demo

checksums:
	@sha256sum resources/en/*.txt

# Gate 1 and the automated parts of Gate 2. The honesty and provenance gates
# are human review by design and are not automatable.
gate: fmt-check lint test
	@echo "--- dependency count (must be zero) ---"
	@cargo tree --depth 1 | tail -n +2 | grep -v '^$$' || echo "   none"
	@echo "--- nondeterministic collections in source (justify each hit) ---"
	@! grep -rn "HashMap\|HashSet" src || true
	@echo "--- clock, rng, network, env in source (must be empty) ---"
	@! grep -rn "SystemTime\|Instant::now\|rand::\|std::env::var\|TcpStream" src
	@echo "gate: automated checks complete; now walk RELEASE_GATES_CHECKLIST.md"

clean:
	cargo clean
