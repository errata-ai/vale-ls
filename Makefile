all: build

release:
	git tag $(tag)
	git push origin $(tag)

build:
	cargo build --release

test:
	cargo test
