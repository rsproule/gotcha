all: 
	cargo run --bin crawl -- --address 0xABc0948e1551c52C7D0Cfc7b9FB2f95a8B2CCF10 --recursive-depth 4 > out/full.txt
	grep -e Edge: out/full.txt > out/edges.txt
	grep -e Node: out/full.txt > out/nodes.txt
	cargo run --bin viz -- --nodes-path out/nodes.txt --edges-path out/edges.txt > out/graph.dot 
	dot -Tsvg out/graph.dot > out/graph.svg
