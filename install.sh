#!/bin/bash

####################################################################################
# Specify the target triplet as first parameter
# You can get a list of the installed targets with `rustup target list --installed`
# Add new targets with `rustup add target <triplet>`
#
# Example: ./install.sh x86_64-unknown-linux-gnu
####################################################################################

if [ "$1" ]; then
  target="$1"
else
  echo "No target specified, taking first rustup installed target instead"
  target="`rustup target list --installed | head -1`"
fi

./build.sh $target
./gradlew --console plain publishToMavenLocal -Ptarget=$target