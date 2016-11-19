# httpsdns

test udp port https://wiki.itadmins.net/network/tcp_udp_ping
sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"
this creates a zero-len udp package, that is used to mock a request

also testable as real dns proxy on linux:
cargo build
sudo RUST_BACKTRACE=1 ./target/debug/httpsdns 0.0.0.0:53
put a new line with "nameserver 127.0.0.1" in /etc/resolf.conf
(obviously: comment out the old with a # or backup it otherwise)

