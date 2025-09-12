#!/bin/bash
set -euo pipefail

PROVISION_URL_BASE="http://172.17.0.1:4242/arista"
FILE_LIST="autoreload.py autoreload.sh autoreload_legacy.sh artnet-bridge.sh"
DEPLOY_SERVER="http://172.17.0.1:4200"

function apply_config() {
	echo "[PROVISION] Applying configuration..."
	(FastCli -p 15 -c  "configure replace ${DEPLOY_SERVER}/by_serial/${SERIAL_NUMBER}" | tee /mnt/flash/provision-configure.log) || return 1
	echo "[PROVISION] Application complete!..."
	if grep -q "error" /mnt/flash/provision-configure.log ; then
		echo "[PROVISION] Detected error in configuration. Exiting..."
		return 1
	fi
	if grep -q "invalid" /mnt/flash/provision-configure.log ; then
		echo "[PROVISION] Detected error in configuration. Exiting..."
		return 1
	fi
	echo "[PROVISION] Saving..."
	FastCli -p 15 -c "write mem"
}

function main() {
	echo "[PROVISION] Entered main."
	SERIAL_NUMBER=$(awk -F" " '/^SerialNumber: / {print $2}' /etc/fdl)
	echo "[PROVISION] Detected serial number: $SERIAL_NUMBER"
	for file in $FILE_LIST; do
		echo "[PROVISION] Fetching file $file to /mnt/flash/$file..."
		curl -o "/mnt/flash/$file" "${PROVISION_URL_BASE}/$file"
	done
	S=0
	for i in 1 2 3 4 5; do
		S=1
		apply_config && break || true
		S=0
		echo "[PROVISION] Retrying failed..."
		sleep 15
	done
	if [[ "$S" == "0" ]]; then
		echo "[PROVISION] Failed to apply config!"
		exit 1
	fi
	echo "[PROVISION] Finished!"
}

main "$@"
