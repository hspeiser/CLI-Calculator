.PHONY: test test-core

test:
	RUST_BACKTRACE=1 cargo test --workspace --all-targets

test-core:
	RUST_BACKTRACE=1 cargo test -p calc_core --lib --tests


