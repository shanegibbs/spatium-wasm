build:
	bash -c "find target -name '*.wasm' |xargs rm -f"
	cargo  +nightly build --target wasm32-unknown-unknown --release
	wasm-gc target/wasm32-unknown-unknown/release/spatium_wasm.wasm html/spatium.wasm

build-debug:
	bash -c "find target -name '*.wasm' |xargs rm -f"
	cargo +nightly build --target wasm32-unknown-unknown
	wasm-gc target/wasm32-unknown-unknown/debug/spatium_wasm.wasm html/spatium.wasm

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
	# https://www.hellorust.com/setup/wasm-target/
	rustup target add wasm32-unknown-unknown --toolchain nightly
	cargo install --git https://github.com/alexcrichton/wasm-gc

run:
	bash -c "cd html && python -m SimpleHTTPServer"
