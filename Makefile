RUST_VERSION=nightly-2018-03-24

build:
	bash -c "find target -name '*.wasm' |xargs rm -f"
	cargo build --target wasm32-unknown-unknown --release
	wasm-gc target/wasm32-unknown-unknown/release/spatium_wasm.wasm docs/spatium_wasm.wasm

build-debug:
	bash -c "find target -name '*.wasm' |xargs rm -f"
	cargo build --target wasm32-unknown-unknown
	# cp \
	    # target/wasm32-unknown-unknown/debug/deps/spatium_wasm.wasm.map \
		# docs
	wasm-gc target/wasm32-unknown-unknown/debug/spatium_wasm.wasm docs/spatium_wasm.wasm

build-watch:
	./scripts/build-watch.sh

test:
	cargo test --all

test-pkg:
	cargo test -p spatium-$(PKG)

test-pkg-nocapture:
	RUST_TEST_THREADS=1 _RUST_BACKTRACE=full cargo test -p spatium-$(PKG) -- --nocapture

test-nocapture:
	cargo test -p spatium-lib -- --nocapture

bench:
	cargo bench --all

bench-pkg:
	cargo bench -p spatium-$(PKG)

tensorflow-bench:
	./tensorflow-benchmark/venv/bin/python ./tensorflow-benchmark/tensorflow-benchmark.py

setup:
	rustup override set $(RUST_VERSION)
	rustup target add wasm32-unknown-unknown --toolchain $(RUST_VERSION)

setup-tools:
	cargo install --force --git https://github.com/alexcrichton/wasm-gc

run:
	bash -c "cd docs && python -m SimpleHTTPServer"
