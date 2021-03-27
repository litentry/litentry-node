HOST1=localhost
RPC_PORT1=9901
curl http://$HOST1:9901 -H "Content-Type:application/json;charset=utf-8" -d "@node01-aura.json"
curl http://$HOST1:9901 -H "Content-Type:application/json;charset=utf-8" -d "@node01-gran.json"

HOST2=localhost
RPC_PORT2=9903
curl http://$HOST2:9903 -H "Content-Type:application/json;charset=utf-8" -d "@node02-aura.json"
curl http://$HOST2:9903 -H "Content-Type:application/json;charset=utf-8" -d "@node02-gran.json"
