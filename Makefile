.PHONY: web web-debug serve web-test

serve:
	./syntxt-web-wasm/serve.py

web:
	wasm-pack build --target web --release syntxt-web-wasm

web-debug:
	wasm-pack build --target web --dev syntxt-web-wasm

web-test:
	wasm-pack test --firefox --headless syntxt-web-wasm
