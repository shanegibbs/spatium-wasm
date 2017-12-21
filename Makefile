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
	cargo test -p spatium-$(PKG) -- --nocapture

test-nocapture:
	cargo test -p spatium-lib -- --nocapture

setup:
	# https://www.hellorust.com/setup/wasm-target/
	rustup target add wasm32-unknown-unknown --toolchain nightly
	cargo install --git https://github.com/alexcrichton/wasm-gc

run:
	bash -c "cd html && python -m SimpleHTTPServer"
