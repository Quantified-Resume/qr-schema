#!/bin/bash

FOLDER=$(
    cd "$(dirname "$0")/.."
    pwd
)
TARGET_PATH="${FOLDER}/qqq"

tar -zcvf ${TARGET_PATH} \
    --exclude=target/ \
    --exclude=.git/ \
    --exclude=qqq \
    ./
