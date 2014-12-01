#
# Make to Cargo wrappers
#
# So you can issue `make` and project will compile, or `make test` and will compile and issue
#
.PHONY: run test build doc clean
run build:
	cargo $@

test doc clean:
	cargo $@
