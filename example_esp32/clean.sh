#!/bin/sh
set -e
basedir="$(realpath "$0" | xargs dirname)"
cd "$basedir"
cargo +esp clean
rm -rf ".embuild"
