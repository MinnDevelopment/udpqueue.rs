#!/bin/bash

####################################################################################
# Specify the target triplet as first parameter
# You can get a list of the installed targets with `rustup target list --installed`
# Add new targets with `rustup add target <triplet>`
#
# Example: ./install.sh x86_64-unknown-linux-gnu
####################################################################################

./build.sh $1
./gradlew publishToMavenLocal -Dtarget=$1