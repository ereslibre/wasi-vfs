all: run

some-test.wasm: main.c
	clang -o $@ $<

.PHONY: run
run: some-test.wasm
	wasmtime --mapdir .::. $<

.PHONY: clean
clean:
	rm -f *.wasm
