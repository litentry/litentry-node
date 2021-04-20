#!/bin/bash

ECHO="echo"
if [ `uname` = "Darwin" ]; then
    echo "MacOSX system"
    ECHO="echo"
elif [ `uname` = "Linux" ]; then
    echo "Linux system"
    ECHO="echo -e"
fi

EXECUTOR=
BINARY=litentry-node
CHAIN_SPEC=litentry

# 1. Locate project workspace
SCRIPT_DIR="$(cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P)"
CWD=$(dirname $SCRIPT_DIR)

# 2. Determine exector, prefer to execute release version
if [[ -f $CWD/target/release/$BINARY ]]
then
    EXECUTOR=$CWD/target/release/$BINARY
elif [[ -f $CWD/target/debug/$BINARY ]]
then
    EXECUTOR=$CWD/target/debug/$BINARY
else
    $ECHO "No available binary found. Exiting..."
    exit 1
fi

# 2.1 Check *rust* env
. $SCRIPT_DIR/check-rust-env.sh || exit 1

# 3. Execute
$ECHO "Exector: $EXECUTOR"

stopNodes() {
    local numOfProcess=-1
    while [ "$numOfProcess" -ne "0" ]; do
        echo "Killing $BINARY ..."
        pkill $BINARY
        sleep 1
        numOfProcess=`ps aux | grep $BINARY  | grep -v grep | wc -l`
    done
}
# stop all nodes
stopNodes

getip() {
    local ip=
    interfaces=(en0 eth0)
    for interface in ${interfaces[@]}
    do

        valid_interface=`ifconfig $interface &> /dev/null`
        if [ "$?" == "0" ]
        then
            ip=`ifconfig $interface | grep "inet " | awk '{print $2}'`
            break
        fi
    done
    $ECHO $ip
}

ip=$(getip)

colorText() {
    text=$1
    NC='\033[0m'
    color='\033[0;32m'
    $ECHO "${color}${text}${NC}"
}

colorText "Starting node 01 ..."
$EXECUTOR --chain litentry --rpc-external --ws-external --rpc-methods Unsafe --rpc-cors all --ws-port 9900 --port 30334 --rpc-port 9901 --validator -d /tmp/1 &> /dev/null &

sleep 3

node_identity=`curl -s http://$ip:9901 -H "Content-Type:application/json;charset=utf-8" -d '{ "jsonrpc": "2.0", "id": 1, "method": "system_localPeerId",  "params": [] }' | jq -r '.result'`
colorText "Node identity of node 01: $node_identity"

colorText "Starting node 02 ..."
$EXECUTOR --chain litentry --rpc-external --ws-external --rpc-methods Unsafe --rpc-cors all --ws-port 9902 --port 30335 --rpc-port 9903 --validator -d /tmp/2 --bootnodes /ip4/$ip/tcp/30334/p2p/$node_identity &> /dev/null &

sleep 3

colorText "Running nodes:"
ps aux|grep $BINARY | grep -v grep
