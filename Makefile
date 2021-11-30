debug:
	cargo build
	ln -sf target/debug/pkg-warner .
release:
	cargo build --release
	ln -sf target/release/pkg-warner .
clean:
	rm -rf target/ Cargo.lock pkg-warner
install:
	cargo build --release
	sudo cp target/release/pkg-warner /usr/bin/pkg-warner
	sudo touch /etc/package_managers.toml
