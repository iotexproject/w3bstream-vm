.PHONY: risc0_depends
risc0_depends:
  cargo install cargo-binstall
  echo yes | cargo binstall cargo-risczero
  cargo risczero install

.PHONY: test
test:
	cargo test -- --test-threads=1

