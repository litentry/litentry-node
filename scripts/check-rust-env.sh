#!/bin/bash

prompt_message=true

prompt() {
    if [[ "$prompt_message" -eq "true" ]]; then
        echo "Please following the following instruction to setup *rust* environment"
        echo "https://substrate.dev/docs/en/knowledgebase/getting-started"
        prompt_message=false
    fi
}

check_rust() {
    echo "Checking rustup..."
    found=`which rustup | wc -l`
    if [ "$found" -ne "1" ]; then
        echo "Not found rustup"
        prompt
        return 1
    else
        rustup check
    fi
}


check_wasm() {
    echo "Checking wasm32-unknown-unknown ..."
    found=`rustup target list | grep wasm32-unknown-unknown | grep installed | wc -l`
    if [ "$found" -ne "1" ]; then
        echo "Not found wasm32-unknown-unknown"
        prompt
        return 1
    else
        echo "Found wasm32-unknown-unknown"
    fi
}


check_rust && check_wasm
