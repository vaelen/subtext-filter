all: target/release/subtext-filter

clean:
	cargo clean

target/release/subtext-filter: src/main.rs
	cargo build --release

install: target/release/subtext-filter
	cp target/release/subtext-filter /usr/local/bin/subtext-filter
	chmod go-x /usr/local/bin/subtext-filter
	chmod u+x /usr/local/bin/subtext-filter
	cp subtext-filter.service /etc/systemd/system/subtext-filter.service
	systemctl daemon-reload
	systemctl enable subtext-filter
	systemctl start subtext-filter
