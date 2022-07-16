
# udpqueue.rs

This is a rust implementation of the original JDA-NAS natives. This can be used to make a minimal modular jar with only the required target natives.


## Setup

Right now this only supports **Linux x86-64** builds. I plan to set up github actions to publish more targets in the future.

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
    implementation("club.minnced:udpqueue-native-linux-x86-64:0.1.1")
}
```

Alternatively, you can also install rustup locally on your target platform and build it yourself.

Use `./install.sh <triplet>` to install the jar for your specific platform in maven local. Example: `./install.sh x86_64-unknown-linux-gnu`