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

# benchmark build
build-benchmark:
	cd node; cargo build --features runtime-benchmarks --release
	cd runtime; cargo build --features runtime-benchmarks --release
	cd pallets/offchain-worker; cargo build --features runtime-benchmarks --release
	cd pallets/account-linker; cargo build --features runtime-benchmarks --release

benchmark-account-linker:
	target/release/litentry-node benchmark \
	--chain dev \
	--execution=wasm  \
	--wasm-execution=compiled \
	--pallet account-linker \
	--extrinsic do_something \
	--steps 20 \
	--repeat 50

benchmark-offchain-worker:
	target/release/litentry-node benchmark \
	--chain dev \
	--execution=wasm  \
	--wasm-execution=compiled \
	--pallet offchain-worker \
	--extrinsic asset_claim \
	--steps 20 \
	--repeat 50

fmt:
	cargo fmt
define pkgid
	$(shell cargo pkgid $1)
endef
