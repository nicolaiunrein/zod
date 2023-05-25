#!/bin/bash
set -e;
TEST=$1;

cargo watch -s "cargo expand --test $TEST test > ./tests/expanded/$TEST.rs" --ignore ./expanded 

