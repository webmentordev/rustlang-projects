#!/bin/bash
FILE="./profile.json"

cd ui && npx nuxi generate && cd .. 

if [ ! -f "$FILE" ]; then
    cat ./profile.example.json > "$FILE"
fi

cargo run