[![publish-natives](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/publish.yml/badge.svg)](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/publish.yml)
[![rust-clippy analyze](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/MinnDevelopment/udpqueue.rs/actions/workflows/rust-clippy.yml)

# udpqueue.rs

This is a rust implementation of the original JDA-NAS natives. This can be used to make a minimal modular jar with only the required target natives.


## Setup

[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-api?color=blue&label=udpqueue-api) ](https://search.maven.org/artifact/club.minnced/udpqueue-api)

Supported native platforms:

Linux x86 (intel):

[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-x86-64?color=blue&label=linux-x86-64&logo=linux&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-x86-64)
[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-x86?color=blue&label=linux-x86&logo=linux&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-x86)

Linux ARM (v7 and x64):

[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-aarch64?color=blue&label=linux-aarch64&logo=linux&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-aarch64)
[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-linux-arm?color=blue&label=linux-arm&logo=linux&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-linux-arm)

Windows x86 (intel):

[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-win-x86-64?color=blue&label=win-x86-64&logo=windows&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-win-x86-64)
[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-win-x86?color=blue&label=win-x86&logo=windows&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-win-x86)

MacOS/Darwin universal (x86 intel & aarch64 M1):

[ ![](https://img.shields.io/maven-central/v/club.minnced/udpqueue-native-darwin?color=blue&label=darwin&logo=apple&logoColor=white) ](https://search.maven.org/artifact/club.minnced/udpqueue-native-darwin)

More platforms can be added on request. Linux shared libraries are compiled against **GLIBC 2.25**.

While this project is published to maven-central, the lavaplayer commons dependency is currently only available through jcenter. So you will have to depend on jcenter for now.

1. Add the original [jda-nas](https://github.com/sedmelluq/jda-nas) dependency to your project, and exclude `udp-queue` from its transitive dependencies:

```gradle
repositories {
    mavenCentral()
    jcenter()
}

dependencies {
    implementation("com.sedmelluq:jda-nas:1.1.0") {
        exclude(module="udp-queue")
    }
}
```

2. Add udpqueue natives

```gradle
dependencies {
    // Fully modular, choose which platforms to use!
    implementation("club.minnced:udpqueue-native-linux-x86-64:0.2.0") // adds linux 64bit
    implementation("club.minnced:udpqueue-native-win-x86-64:0.2.0") // adds windows 64bit
}
```

Alternatively, you can also install rustup locally on your target platform and build it yourself.

Use `./install.sh <triplet>` to install the jar for your specific platform in maven local. Example: `./install.sh x86_64-unknown-linux-gnu`

To add all supported platforms, you can use this:

```gradle
dependencies {
    implementation("club.minnced:udpqueue-native-linux-x86-64:0.2.0")
    implementation("club.minnced:udpqueue-native-linux-x86:0.2.0")
    implementation("club.minnced:udpqueue-native-linux-aarch64:0.2.0")
    implementation("club.minnced:udpqueue-native-linux-arm:0.2.0")
    implementation("club.minnced:udpqueue-native-win-x86-64:0.2.0")
    implementation("club.minnced:udpqueue-native-win-x86:0.2.0")
    implementation("club.minnced:udpqueue-native-darwin:0.2.0")
}
```
