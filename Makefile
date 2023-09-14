define with_path
    LD_LIBRARY_PATH=/home/brent/mambaforge/lib $1
endef

run:
	$(call with_path, cargo run)

clippy:
	$(call with_path, cargo clippy)
