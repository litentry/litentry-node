all:
	@echo "Make All"

build:
	cargo build
node:
	cargo build --package $(call pkgid, litentry-node)
runtime:
	cargo build --package $(call pkgid, litentry-runtime)
offchain-worker:
	cargo build --package $(call pkgid, pallet-offchain-worker)
account-linker:
	cargo build --package $(call pkgid, pallet-account-linker)
litentry-token-server:
	cargo build --package $(call pkgid, litentry-token-server)

test-node:
	cargo test --package $(call pkgid, litentry-node)
test-runtime:
	cargo test --package $(call pkgid, litentry-runtime)
test-account-linker:
	cargo test --package $(call pkgid, pallet-account-linker)
test-offchain-worker:
	cargo test --package $(call pkgid, pallet-offchain-worker)
test-litentry-token-server:
	cargo test --package $(call pkgid, litentry-token-server)

test:
	cargo test

fmt:
	cargo fmt
define pkgid
	$(shell cargo pkgid $1)
endef
