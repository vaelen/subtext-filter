# subtext-filter
A firewall manager for Subtext BBS.

Currently the only supported firewall is nftables.

Edit main.rs if you need to change the udp port or the cache timeout.

To build:
```
$ cargo build --release
```

or:
```
$ make
```

An example systemd service file can be found in `subtext-filter.service`.

Full install, assuming systemd and default paths:
```
$ make
$ sudo make install
```
