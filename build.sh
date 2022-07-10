#!/bin/bash

####################################################################################
# Specify the target triplet as first parameter
# You can get a list of the installed targets with `rustup target list --installed`
# Add new targets with `rustup add target <triplet>`
#
# Example: ./build.sh x86_64-unknown-linux-gnu
####################################################################################

target=$1

echo "Building for target: $target"

pushd native
cargo build -r --target=$target
popd

chmod u+x gradlew
./gradlew --console plain build -Ptarget=$target
