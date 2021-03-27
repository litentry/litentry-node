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

# 3. Execute
echo "Exector: $EXECUTOR"

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
    echo $ip
}

ip=$(getip)
# echo "IP: $ip"
echo "Starting node 01 ..."
$EXECUTOR --chain litentry --rpc-external --ws-external --rpc-methods Unsafe --rpc-cors all --ws-port 9900 --port 30334 --rpc-port 9901 --validator &> /dev/null &

seconds=5
echo "Waiting $seconds seconds..."
sleep $seconds

node_identity=`curl http://$ip:9901 -H "Content-Type:application/json;charset=utf-8" -d '{ "jsonrpc": "2.0", "id": 1, "method": "system_localPeerId",  "params": [] }' | jq -r '.result'`
echo "Node identity: $node_identity"

echo "Starting node 02..."
$EXECUTOR --chain litentry --rpc-external --ws-external --rpc-methods Unsafe --rpc-cors all --ws-port 9902 --port 30335 --rpc-port 9903 --validator -d /tmp/1 --bootnodes /ip4/$ip/tcp/30334/p2p/$node_identity &> /dev/null &

seconds=5
echo "Waiting $seconds seconds..."
sleep $seconds
