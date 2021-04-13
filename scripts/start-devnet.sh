#!/bin/bash

EXECUTOR=
BINARY=litentry-node

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
    echo "No available binary found. Exiting..."
    exit 1
fi

# 2.1 Check *rust* env
. $SCRIPT_DIR/check-rust-env.sh || exit 1

# 3. Execute
echo "Exector: $EXECUTOR"

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

echo "Starting dev node ..."
$EXECUTOR --tmp --dev --rpc-external --ws-external --rpc-methods Unsafe --rpc-cors all --alice
