build-app:
	cargo build --release
run-app:
	cargo run --release

build-recorder:
	cargo build --no-default-features --features record
run-recorder:
	cargo run --release --no-default-features --bin neothesia-cli --features record -- $(file)