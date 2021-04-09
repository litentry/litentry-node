SCRIPT_DIR="$(cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P)"

HOST1=localhost
RPC_PORT1=9901
curl http://$HOST1:9901 -H "Content-Type:application/json;charset=utf-8" -d "@$SCRIPT_DIR/node01-aura.json"
curl http://$HOST1:9901 -H "Content-Type:application/json;charset=utf-8" -d "@$SCRIPT_DIR/node01-gran.json"

HOST2=localhost
RPC_PORT2=9903
curl http://$HOST2:9903 -H "Content-Type:application/json;charset=utf-8" -d "@$SCRIPT_DIR/node02-aura.json"
curl http://$HOST2:9903 -H "Content-Type:application/json;charset=utf-8" -d "@$SCRIPT_DIR/node02-gran.json"
