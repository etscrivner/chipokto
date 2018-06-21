.PHONY: clean test

all:
	@cargo build

test:
	@cargo test

release:
	@cargo build --release

clean:
	@find . -name "*~" -delete
	@rm -rf target okto/target oktodis/target chipokto/target
