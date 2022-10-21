.PHONY: clean default test

default:
	cargo build --release

clean:
	rm -rf target/

test:
	cargo test --all
	cargo clippy --all --tests --examples
