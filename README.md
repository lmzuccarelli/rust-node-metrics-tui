# Overview

Simple promtheus node metrics tui viewer written in Rust

It scrapes at configured intervals (only the highlighted server to limit bandwidth)

Metrics viewed

- cpu
- memory (ram)
- network (uploaded and downloaded bytes)
- disk (mounted volumes)
- server info (via popup)

## Usage

clone this repo

```
git clone https://github.com/lmzuccarelli/rust-node-metrics-tui

cd rust-node-metrics-tui
```

### Build binary

This assumes you have installed Rust


```
make build
```

## Workflow

- Update the specific addresses as required for your setup (in the ./script/infrastructure.sh file)

- Update the "servers" field in the configs/rust-node-metrics-tui.json file

- Create the service systemd config (execute the following script)

```bash
./scripts/infrastructure.sh create_configs
```

- Get the node_exporter binary and untar to projects bin directory (change for specific linux version if needed)

```bash
./scripts/infrastructure.sh get_node_exporter
```

- Deploy the prometheus node_exporter binary to all servers

```bash
./scripts/infrastructure.sh deploy_service
```

- Start the service on each node

```bash
./scripts/infrastructure.sh start_service
```

- Test the endpoints by using curl (change "server" for each node you have deployed to)

```bash
curl http://<server>:9100/metrics
```

- Launch the tui

```bash
./target/release/examples/rust-node-metrics-tui --config config/rust-node-metrics-tui.json
```

## Screenshot

![image](assets/screenshot.jpg)

