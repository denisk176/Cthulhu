#!/bin/bash
set -exuo pipefail
cargo build --release --target x86_64-unknown-linux-musl
ssh root@10.42.0.22 systemctl stop cthulhu
scp target/x86_64-unknown-linux-musl/release/cthulhu root@10.42.0.22:/home/root/cthulhu/
ssh root@10.42.0.22 systemctl start cthulhu
