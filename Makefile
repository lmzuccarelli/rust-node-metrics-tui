.PHONY: all clean-all build 

all: clean-all build

build: 
	cargo build --release

clean-all:
	rm -rf cargo-test*
	cargo clean
	rm -rf ./target/debug
