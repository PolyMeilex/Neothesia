build-app:
	cargo build --release --bin neothesia
run-app:
	cargo run --release --bin neothesia

install-app:
	cargo install --path neothesia

check-recorder:
	cargo check -p neothesia-cli
build-recorder:
	cargo build --release -p neothesia-cli
run-recorder:
	cargo run --release -p neothesia-cli -- $(file)

