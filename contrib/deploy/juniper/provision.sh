#!/bin/sh
PROVISION_URL_BASE="http://172.17.0.1:4242/juniper"
FILE_LIST="autoreload.cron autoreload.sh"
DEPLOY_SERVER="http://172.17.0.1:4200"

CONFIG_FILE=/tmp/provision.config
load_file=/tmp/provision.load
check_file=/tmp/provision.check

pfail() {
	echo "[PROVISION] Failure!"
	exit 1
}

echo '[PROVISION] Provisioning start!'

echo "[PROVISION] Disabling VC ports..."
echo "show virtual-chassis vc-port" | cli | grep -i Configured | (while read line ; do
    pic=`echo "$line" | awk '{ print $1 }' | awk -F/ '{ print $1 }'`
    port=`echo "$line" | awk '{ print $1 }' | awk -F/ '{ print $2 }'`
    echo "request virtual-chassis vc-port delete pic-slot $pic port $port" | cli
done)
echo "[PROVISION] Attempting to configure VCP ports to act as network ports..."
echo "request virtual-chassis mode network-port" | cli
echo '[PROVISION] VCP ports complete!'

for f in $FILE_LIST ; do
	echo "[PROVISION] Retrieving $f..."
	fetch -o /var/tmp/$f "$PROVISION_URL_BASE/$f" || pfail
done
chmod +x /var/tmp/autoreload.sh

SERIALNO=`sysctl hw.chassis.serialid | awk '{ print $2 }'`
echo "[PROVISION] Detected serial number: $SERIALNO"

S=0
for i in 1 2 3 4 5; do
	S=1
	echo "[PROVISION] Fetching config file..."
	fetch -o $CONFIG_FILE "$DEPLOY_SERVER/by_serial/$SERIALNO" && break || true
	S=0
	echo "[PROVISION] Retrying failed..."
	sleep 15
done
if [ "$S" -eq "0" ]; then
	echo "[PROVISION] Failed to apply config!"
	pfail
fi

chmod 644 $CONFIG_FILE

echo "[PROVISION] Checking config file..."
# check if the config has errors
{ echo "configure"
  echo "load override $CONFIG_FILE | save $load_file"
  echo "commit check | save $check_file"
  echo "rollback 0"
  echo "exit"
} | /usr/sbin/cli

if grep -q error $load_file; then
	echo "[PROVISION] Failure in load file!"
	cat $load_file
	pfail
fi

if grep -q error $check_file; then
	echo "[PROVISION] Failure in check file!"
	cat $check_file
	pfail
fi

echo "[PROVISION] Applying config file..."

{ echo "configure"
  echo "load override $CONFIG_FILE | match IDONOTWANTYOUROUTPUT"
  echo "commit and-quit"
} | /usr/sbin/cli

{ echo "request system configuration rescue save"
  echo "exit"
} | /usr/sbin/cli

echo "[PROVISION] Enabling crontab..."
crontab /var/tmp/autoreload.cron
crontab -l

echo '[PROVISION] Provisioning complete!'
