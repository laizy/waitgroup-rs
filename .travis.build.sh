#!/bin/bash
set -e
set -x

if rustup component add clippy;
then
	cargo clippy --all ;
else
	echo 'Skipping clippy';
fi

cargo build --release 
cargo test

