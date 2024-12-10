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

Example output from systemd log:
```
Dec 11 02:00:00 vaelen systemd[1]: Started Automatic Firewall Management for Subtext BBS.
Dec 11 02:00:00 vaelen subtext-filter[12336]: Loaded 0 blocked IPs
Dec 11 02:00:00 vaelen subtext-filter[12336]: Listening on 0.0.0.0:1234
Dec 11 02:00:10 vaelen subtext-filter[12336]: Blocking 222.138.16.136
Dec 11 02:00:40 vaelen subtext-filter[12336]: Blocking 223.8.187.192
Dec 11 02:02:34 vaelen subtext-filter[12336]: Blocking 109.162.15.4
Dec 11 02:05:11 vaelen subtext-filter[12336]: Handles: 3
Dec 11 02:05:11 vaelen subtext-filter[12336]: Removing block on 222.138.16.136 after 5 minutes (Handle: 169)
Dec 11 02:05:41 vaelen subtext-filter[12336]: Handles: 2
Dec 11 02:05:41 vaelen subtext-filter[12336]: Removing block on 223.8.187.192 after 5 minutes (Handle: 170)
```
