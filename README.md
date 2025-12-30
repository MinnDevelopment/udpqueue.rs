[![publish-natives](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/publish.yml/badge.svg)](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/publish.yml)
[![build](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/build.yml/badge.svg)](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/build.yml)
[![rust-clippy analyze](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/rust-clippy.yml)

# udpqueue.rs

This is a rust implementation of the original JDA-NAS natives. This can be used to make a minimal modular jar with only the required target natives.


## Setup

[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-api?color=blue&label=udpqueue-api)](https://search.maven.org/artifact/club.minnced/udpqueue-api)

Supported native platforms:

Linux x86 (intel):

[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-x86-64?color=blue&label=linux-x86-64&logo=linux&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-x86-64)
[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-x86?color=blue&label=linux-x86&logo=linux&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-x86)
[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-musl-x86-64?color=blue&label=linux-musl-x86-64&logo=linux&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-musl-x86)

Linux ARM (v7 and x64):

[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-aarch64?color=blue&label=linux-aarch64&logo=linux&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-aarch64)
[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-arm?color=blue&label=linux-arm&logo=linux&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-arm)
[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-musl-aarch64?color=blue&label=linux-musl-aarch64&logo=linux&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-musl-aarch64)

Windows x86 (intel):

[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-win-x86-64?color=blue&label=win-x86-64&logo=windows&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-win-x86-64)
[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-win-x86?color=blue&label=win-x86&logo=windows&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-win-x86)

Windows aarch64:

[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-win-aarch64?color=blue&label=win-aarch64&logo=windows&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-win-aarch64)

MacOS/Darwin universal (x86 intel & aarch64 M1):

[![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-darwin?color=blue&label=darwin&logo=apple&logoColor=white)](https://search.maven.org/artifact/club.minnced/udpqueue-native-darwin)

More platforms can be added on request. Linux shared libraries are compiled against **GLIBC 2.27**.

Simply install the version of `udpqueue-native-*` for your platform:

```gradle
repositories {
    mavenCentral()
}

dependencies {
    // Fully modular, choose which platforms to use!
    implementation("club.minnced:udpqueue-native-linux-x86-64:0.2.12") // adds linux 64bit
    implementation("club.minnced:udpqueue-native-win-x86-64:0.2.12") // adds windows 64bit
}
```

Alternatively, you can also install rustup locally on your target platform and build it yourself.

Example:

```
rustup target add x86_64-unknown-linux-gnu
./gradlew publishToMavenLocal -Ptarget=x86_64-unknown-linux-gnu
```

To add all supported platforms, you can use this:

```gradle
repositories {
    mavenCentral()
}

dependencies {
    implementation("club.minnced:udpqueue-native-linux-x86-64:0.2.12")
    implementation("club.minnced:udpqueue-native-linux-x86:0.2.12")
    implementation("club.minnced:udpqueue-native-linux-aarch64:0.2.12")
    implementation("club.minnced:udpqueue-native-linux-arm:0.2.12")
    implementation("club.minnced:udpqueue-native-linux-musl-x86-64:0.2.12")
    implementation("club.minnced:udpqueue-native-linux-musl-aarch64:0.2.12")
    implementation("club.minnced:udpqueue-native-win-x86-64:0.2.12")
    implementation("club.minnced:udpqueue-native-win-x86:0.2.12")
    implementation("club.minnced:udpqueue-native-win-aarch64:0.2.12")
    implementation("club.minnced:udpqueue-native-darwin:0.2.12")
}
```
