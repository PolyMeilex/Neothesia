build-app:
	cargo build --release --bin neothesia
run-app:
	cargo run --release --bin neothesia

install-app:
	cargo install --path neothesia

build-icons:
	rm -rf target/neothesia-icons
	SOURCE_DATE_EPOCH=0 uvx --from nanoemoji==0.16.0 nanoemoji \
		--color_format glyf \
		--family "Neothesia Icons" \
		--output_file neothesia-icons.ttf \
		--upem 1000 \
		--ascender 850 \
		--descender=-150 \
		--width 1000 \
		--version_major 1 \
		--version_minor 1 \
		--build_dir target/neothesia-icons \
		assets/fonts/neothesia-icons-src/*.svg
	cp target/neothesia-icons/neothesia-icons.ttf assets/fonts/neothesia-icons.ttf

check-recorder:
	cargo check -p neothesia-cli
build-recorder:
	cargo build --release -p neothesia-cli
run-recorder:
	cargo run --release -p neothesia-cli -- $(file)

