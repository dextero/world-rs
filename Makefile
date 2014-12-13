VERBOSE ?= 0
FLAGS ?=

ifeq ($(VERBOSE),1)
FLAGS += --verbose
endif

default:
	cargo build $(FLAGS)

release:
	cargo build --release $(FLAGS)

clean:
	cargo clean $(FLAGS)
