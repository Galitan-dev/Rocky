all:
	test production

test:
	cargo test -q

production:
	cargo build --release
	strip target/release/rocky
	mv target/release/rocky /usr/local/bin/
	chmod ugo+x /usr/local/bin/

dev:
	cargo build
	mv target/debug/rocky /usr/local/bin/
	chmod ugo+x /usr/local/bin/
