ws-serve: 
	websocat -t ws-l:127.0.0.1:1234 broadcast:mirror:

example: 
	cargo run --bin crawl -- --address 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045  \
		--recursive-depth 2 \
		--backward false \
		| websocat ws://127.0.0.1:1234

