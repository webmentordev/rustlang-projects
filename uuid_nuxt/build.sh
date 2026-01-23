#!/bin/bash

cd ui && npm install && npx nuxi generate && cd .. && cargo build --release