all: run run-empty-vfs

some-test.wasm: main.c
	$(CC) -o $@ $<

some-test-with-vfs.wasm: main.c ../target/wasm32-unknown-unknown/debug/libwasi_vfs.a
	$(CC) -o $@ ../target/wasm32-unknown-unknown/debug/libwasi_vfs.a $<

.PHONY: run
run: some-test.wasm
	nix run github:ereslibre/nixities#wasmtime -- --mapdir .::. $<

.PHONY: run-empty-vfs
run-empty-vfs: some-test-with-vfs.wasm
	nix run github:ereslibre/nixities#wasmtime -- --mapdir .::. $<

.PHONY: clean
clean:
	rm -f *.wasm

../target/wasm32-unknown-unknown/debug/libwasi_vfs.a:
	make -C .. target/wasm32-unknown-unknown/debug/libwasi_vfs.a
