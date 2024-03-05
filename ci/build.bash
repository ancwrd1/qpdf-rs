#!/usr/bin/env bash
# Script for building your rust projects.
set -e

source ci/common.bash

# $1 {path} = Path to cross/cargo executable
CROSS=$1
# $2 {string} = <Target Triple>
TARGET_TRIPLE=$2
# $3 {boolean} = Whether to use vendored sources.
VENDOR=$3
# $4 {boolean} = Whether or not building for release or not.
RELEASE_BUILD=$4

required_arg $CROSS 'CROSS'
required_arg $TARGET_TRIPLE '<Target Triple>'

if [ -n "$VENDOR" ]; then
    VENDOR="--features vendored"
fi

if [ -z "$RELEASE_BUILD" ]; then
    $CROSS build --target $TARGET_TRIPLE $VENDOR --workspace
    $CROSS build --target $TARGET_TRIPLE --all-features --workspace
else
    $CROSS build --target $TARGET_TRIPLE --all-features --release --workspace
fi

