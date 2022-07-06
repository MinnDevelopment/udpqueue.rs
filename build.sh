#!/bin/bash

####################################################################################
# Specify the target triplet as first parameter
# You can get a list of the installed targets with `rustup target list --installed`
# Add new targets with `rustup add target <triplet>`
#
# Example: ./build.sh x86_64-unknown-linux-gnu
####################################################################################

cargo build -r --target=$1

pushd java
chmod u+x gradlew
./gradlew build -Dtarget=$1
popd