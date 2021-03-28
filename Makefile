.PHONY: web

web:
	wasm-pack build --target web --release syntxt-web-wasm
