#!/bin/bash

read -p "Enter mnemonic: " SECRET
echo
read -p "Enter public key: " KEY
echo

ENDPOINT="http://localhost:9933"
NETWORK="litentry"



curl "$ENDPOINT" -H "Content-Type:application/json;charset=utf-8" -d \
    """{
    \"jsonrpc\":\"2.0\",
    \"id\":1,
    \"method\":\"author_insertKey\",
    \"params\": [
        \"ocw!\",
        \"${SECRET}\",
        \"${KEY}\"
    ]
    }"""

