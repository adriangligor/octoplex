#!/bin/bash
set -e

if [ "$1" = 'octoplex' ]; then
    shift
    exec ./bin/octoplex "$@"
elif [ "$1" = 'octoplex-dev' ]; then
    shift
    exec cargo run --frozen "$@"
elif [ "$1" = 'octoplex-test' ]; then
    shift
    exec cargo test --frozen "$@"
fi

exec "$@"
