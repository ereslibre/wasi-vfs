all: run run-empty-vfs

some-test.wasm: main.c
	clang -o $@ $<

some-test-with-vfs.wasm: main.c ../target/wasm32-unknown-unknown/debug/libwasi_vfs.a
	clang -o $@ ../target/wasm32-unknown-unknown/debug/libwasi_vfs.a $<

.PHONY: run
run: some-test.wasm
	wasmtime --mapdir .::. $<

.PHONY: run-empty-vfs
run-empty-vfs: some-test-with-vfs.wasm
	wasmtime --mapdir .::. $<

.PHONY: clean
clean:
	rm -f *.wasm
