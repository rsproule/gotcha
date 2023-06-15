# Gotcha

<p align="center">
  <img width="1000" src="graph-demo.gif">
</p>

A script for fetching all the counter-parties to a given ethereum address.

Relies on:

- etherscan
- metadock api

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
export ETHERSCAN_API_KEY=<your etherscan api key> // since we rely on ol etherscan for labels and some other shit 
cargo run --bin crawl -- --address <address> | websocat ws://localhost:1234
```

----

## TODO

Frontend explorer features:

- [ ] highlight the path back to the origin address on hover. give ability to export.
- [ ] give a way to export the graph with sharable bundle (just export the graph data object to json).
- [ ] add a way to search the graph for a given address. Highlight that node
- [ ] add a way to add custom labels, with custom colors etc
