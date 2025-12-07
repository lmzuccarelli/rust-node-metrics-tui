#!/bin/bash

# set -x

# This script does not deploy this application. 
# Instead it is used to install and setup the prometheus node_export to a list of given servers
# Ensures the node_exporter binary for amd64 and arm are downloaded and 'untarred'
# Copies the binary to each server and setups the systemd service

USER="${REMOTE_USER:-lzuccarelli}"
PK="${PK_ID:?PK_ID environment variable must be set}"
MS="node-exporter"
DESCRIPTION="A prometheus node metrics exporter"
ARMV7_IP=('192.168.1.209' '192.168.1.222' '192.168.1.125' '192.168.1.76' '192.168.1.202' '192.168.1.149' '192.168.1.230' '192.168.1.203')
AMD64_IP=('192.168.1.185' '192.168.1.221' '192.168.1.45' '192.168.1.169' '192.168.1.62')

create_configs() {
tee config/${MS}.service <<EOF
[Unit]
Description=${MS}

[Service]
ExecStart=/home/${USER}/microservices/node_exporter
Restart=Always
PIDFile=/tmp/${MS}_pid
EOF
}

get_node_exporter() {
  curl -L -s https://github.com/prometheus/node_exporter/releases/download/v1.10.2/node_exporter-1.10.2.linux-amd64.tar.gz -o node_exporter_amd64.tar.gz
  curl -L -s https://github.com/prometheus/node_exporter/releases/download/v1.10.2/node_exporter-1.10.2.linux-armv7.tar.gz -o node_exporter_armv7.tar.gz

  tar -xvf node_exporter_amd64.tar.gz -C ./bin
  tar -xvf node_exporter_armv7.tar.gz -C ./bin

  rm -rf node_exporter_amd64.tar.gz
  rm -rf node_exporter_armv7.tar.gz
}



deploy_service() {

	for host in "${ARMV7_IP[@]}"; do
		ssh -i "${PK}" "${USER}@${host}" -t "mkdir -p /home/${USER}/microservices"
		scp -i "${PK}" "./config/${MS}.service" "${USER}@${host}:/home/${USER}/microservices"
    scp -i "${PK}" "./bin/node_exporter-1.10.2.linux-armv7/node_exporter" "${USER}@${host}:/home/${USER}/microservices"
  	ssh -i "${PK}" "${USER}@${host}" -t "sudo cp /home/${USER}/microservices/${MS}.service /etc/systemd/system/"
	done

  for host in "${AMD64_IP[@]}"; do
		ssh -i "${PK}" "${USER}@${host}" -t "mkdir -p /home/${USER}/microservices"
		scp -i "${PK}" "./config/${MS}.service" "${USER}@${host}:/home/${USER}/microservices"
    scp -i "${PK}" "./bin/node_exporter-1.10.2.linux-amd64/node_exporter" "${USER}@${host}:/home/${USER}/microservices"
    # on fedora ensure SELinux executable has bin_t
		ssh -i "${PK}" "${USER}@${host}" -t "sudo chcon -t bin-t /home/${USER}/microservices/node_exporter"
		ssh -i "${PK}" "${USER}@${host}" -t "sudo cp /home/${USER}/microservices/${MS}.service /etc/systemd/system/"
	done
}

start_service() {
  # example 
  # ssh -i ~/.ssh/id_ed25519-lz lzuccarelli@192.168.1.203 -t "sudo systemctl daemon-reload && sudo systemctl start node-exporter.service"
	for host in "${ARMV7_IP[@]}"; do
	  ssh -i "${PK}" "${USER}@${host}" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
	done

  for host in "${AMD64_IP[@]}"; do
	  ssh -i "${PK}" "${USER}@${host}" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
	done
}

restart_service() {
	for host in "${ARMV7_IP[@]}"; do
	  ssh -i "${PK}" "${USER}@${AR_HOST[3]}" -t "sudo systemctl daemon-reload && sudo systemctl restart ${MS}.service"
	done

	for host in "${AMD64_IP[@]}"; do
	  ssh -i "${PK}" "${USER}@${host}" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
	done
}

stop_service() {
	for host in "${ARMV7_IP[@]}"; do
	  ssh -i "${PK}" "${USER}@${AR_HOST[3]}" -t "sudo systemctl daemon-reload && sudo systemctl stop ${MS}.service"
	done

  for host in "${AMD64_IP[@]}"; do
	  ssh -i "${PK}" "${USER}@${host}" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
	done
}

"$@"
