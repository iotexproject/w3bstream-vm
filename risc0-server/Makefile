integration_test_depends_stop:
	@docker stop postgres_test || true && docker container rm postgres_test || true

.PHONY: integration_test_depends
integration_test_depends: integration_test_depends_stop postgres_test risc0_depends

.PHONY: postgres_test
postgres_test:
	docker run --name postgres_test \
  -e POSTGRES_USER=test_user \
  -e POSTGRES_PASSWORD=test_passwd \
  -e POSTGRES_DB=test \
  -p 15432:5432 \
  -d postgres:14

.PHONY: risc0_depends
risc0_depends:
	cargo install cargo-binstall
	echo yes | cargo binstall cargo-risczero
	curl -L https://foundry.paradigm.xyz | bash
	cargo risczero install

integration_test: integration_test_depends
	cargo test
