RFLAGS="-C link-arg=-s"

all: phoenix

phoenix: contracts/phoenix-bond
	$(call compile_release,phoenix-bond)
	@mkdir -p res
	cp target/wasm32-unknown-unknown/release/phoenix_bond.wasm ./res/phoenix_bond.wasm

# test build

phoenix_test: contracts/phoenix-bond
	$(call compile_test,phoenix-bond)
	cp target/wasm32-unknown-unknown/debug/phoenix_bond.wasm ./res/phoenix_bond_test.wasm

mock_linear: contracts/mock-linear
	$(call compile_release,mock-linear)
	@mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_linear.wasm ./res/mock_linear.wasm

lint:
	cargo fmt -- --check
	cargo clippy --tests -- -D clippy::all

define compile_release
	@rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p $(1) --target wasm32-unknown-unknown --release
endef

define compile_test
	@rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p $(1) --target wasm32-unknown-unknown --features "test"
	@mkdir -p res
endef

test: test-unit test-integration

test-unit: 
	cargo test

TEST_FILE ?= **
LOGS ?=

test-integration: phoenix_test mock_linear
	@mkdir -p ./tests/compiled-contracts/
	@cp ./res/phoenix_bond_test.wasm ./tests/compiled-contracts/
	@cp ./res/mock_linear.wasm ./tests/compiled-contracts/
	NEAR_PRINT_LOGS=$(LOGS) npx ava --timeout=5m tests/__tests__/$(TEST_FILE).ava.ts --verbose
