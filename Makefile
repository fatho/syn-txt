.PHONY: web

web:
	wasm-pack build --target web --release syntxt-web-wasm

web-debug:
	wasm-pack build --target web --dev syntxt-web-wasm
