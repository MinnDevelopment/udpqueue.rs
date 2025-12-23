import udpqueue.gradle.getPlatform
import udpqueue.gradle.targetPlatform

plugins {
    `publishing-environment`
}

publishingEnvironment {
    moduleName = "udpqueue-native-${getPlatform()}"
}

dependencies {
    api(project(":api"))
}

val moveResources by tasks.registering(Copy::class) {
    group = "build"

    from("target/$targetPlatform/release/")

    include {
        it.name == "release" || it.name.endsWith(".so") || it.name.endsWith(".dll") || it.name.endsWith(".dylib")
    }

    into("src/main/resources/natives/${getPlatform()}")
}

val cleanNatives by tasks.registering(Delete::class) {
    group = "build"
    delete(fileTree("src/main/resources/natives"))
}

tasks.named("clean").configure {
    dependsOn(cleanNatives)
}

tasks.processResources {
    dependsOn(moveResources)

    include {
        it.isDirectory || it.file.parentFile.name == getPlatform()
    }
}