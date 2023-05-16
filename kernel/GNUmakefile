# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

PROFILE := release

# Default target.
.PHONY: all
all:
	cargo build --profile $(PROFILE) --target x86_64-unknown-none
	clang -target x86_64-pc-none-elf -static -ffreestanding -nodefaultlibs -nostdlib -fPIC -O2 -T linker.ld -fuse-ld=lld target/x86_64-unknown-none/$(PROFILE)/liblimine_rust_barebones.a -o kernel.elf

# Remove object files and the final executable.
.PHONY: clean
clean:
	cargo clean
	rm -rf kernel.elf

.PHONY: distclean
distclean: clean
