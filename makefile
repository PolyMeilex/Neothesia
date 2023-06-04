build-app:
	cargo build --release
run-app:
	cargo run --release

check-recorder:
	cargo check --no-default-features --features record
build-recorder:
	cargo build --release --no-default-features --features record
run-recorder:
	cargo run --release --no-default-features --bin neothesia-cli --features record -- $(file)