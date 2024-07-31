#!/bin/bash
cargo build --release
sudo setcap cap_net_admin=eip $HOME/rust-advanced/tackle/target/release/tackle
sudo $HOME/rust-advanced/tackle/target/release/tackle &
pid=$!
sudo ip addr add 192.168.0.5/24 dev tun0
sudo ip link set up dev tun0
cleanup() {
    echo "cleaning up.."
    sudo kill $pid 2>/dev/null || true
}
trap 'cleanup' TERM INT
wait $pid