# Gotcha

<p align="center">
  <img width="1000" src="graph-demo.gif">
</p>

Gotcha is a tool to visualize the flow of funds and interactions between accounts on Ethereum.
The rust based cli allows for customizable crawling of the chain and outputs the data in a format
that the simple frontend can consume to render the graph.

Relies on:

- etherscan: contract labels, address counter-party info
- metadock api: address labels

## Usage

Clone the repo:

```bash
git clone git@github.com:rsproule/gotcha.git
```

Some pre-requisites to get the visualization working:

- need to run a local websocket server for the frontend to recieve messages
  - I use websocat: <https://github.com/vi/websocat>
  - `brew install websocat`

Launch the websocket server:

```bash
make ws-server
```

Open the frontend in browser:

```bash
open graph.html
```

Run the crawler script and pipe the output to the websocket server:

```bash
export ETHERSCAN_API_KEY=<your etherscan api key> 
cargo run --bin crawl -- --address <insert an address> | websocat ws://localhost:1234

# more parameters are available 
cargo run --bin crawl -- --help
```

----

## TODO

Frontend explorer features:

- [ ] highlight the path back to the origin address on hover. give ability to export.
- [ ] give a way to export the graph with sharable bundle (just export the graph data object to json).
- [ ] add a way to search the graph for a given address. Highlight that node
- [ ] add a way to add custom labels, with custom colors etc

Crawler features:

- [ ] logging (debug stuff separate from critical output, stdout)
- [ ] serialize the node, like we are doing with edges (simplifies the frontend)
- [ ] alternate search modes (dfs, bfs, etc)
