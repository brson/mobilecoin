#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.
#
# This script is intended to make sample data (bootstrapped ledger) suitable
# for local-networktests.
#
# The data is placed in target/sample_data
# The parameters to the script are:
# - NUM_KEYS
# - NUM_UTXOS_PER_ACCOUNT
#
# The tool attempts to avoid rebootstrapping if it doesn't appear necessary,
# which means:
# - target/sample_data already exists (and conf.json exists)
# - conf.json has the same parameters as the user requrests
#
# The tool reruns bootstrap and records a new conf.json file
# jq must be installed to use the tool

set -e

# Change to the project's root directory
cd $(dirname "$0")/../..
echo "PWD: $PWD"
PROJECT_ROOT=$PWD

# Collect parameters for the bootstrap
SAMPLE_KEYS_NUM=${NUM_KEYS:-1000}
BOOTSTRAP_NUM=${NUM_UTXOS_PER_ACCOUNT:-100}

TARGET="./target/sample_data"

if [ -d $TARGET ]; then
    echo "Found pre-existing sample_data..."
    if [ -f $TARGET/conf.json ]; then
        echo "Found sample_data/conf.json..."
        jq < $TARGET/conf.json

        OLD_NUM_KEYS=$(jq .NUM_KEYS < $TARGET/conf.json)
        OLD_NUM_UTXOS=$(jq .NUM_UTXOS_PER_ACCOUNT < $TARGET/conf.json)
        if [ "$OLD_NUM_KEYS" -eq "$SAMPLE_KEYS_NUM" -a "$OLD_NUM_UTXOS" -eq "$BOOTSTRAP_NUM" ]; then
            echo "Skipping bootstrap"
            exit 0
        else
            echo "Conf has changed, re-boostrapping"
        fi
    else
        echo "No conf.json... re-bootstrapping"
    fi
fi

mkdir -p $TARGET
cd $TARGET

set -x

# We are using if else here because `cargo run` often causes things to be rebuilt unnecessarily,
# because the features will be different if keyfile is the only target (because cargo).
# In workflows where you simply build all, you don't want to reinvoke cargo usually.
if [ -f $PROJECT_ROOT/target/release/sample-keys ]; then
    $PROJECT_ROOT/target/release/sample-keys --num ${SAMPLE_KEYS_NUM} --output-dir keys
else
    cargo run -p keyfile --bin sample-keys --release -- --num ${SAMPLE_KEYS_NUM} --output-dir keys
fi

if [ -f $PROJECT_ROOT/target/release/generate_sample_ledger ]; then
    $PROJECT_ROOT/target/release/generate_sample_ledger --txs ${BOOTSTRAP_NUM}
else
    cargo run --bin generate_sample_ledger --release -- --txs ${BOOTSTRAP_NUM}
fi
