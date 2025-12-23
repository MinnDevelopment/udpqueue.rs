plugins {
    `java-library`
    `maven-publish`
    signing
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(25))
    }
}

subprojects {
    repositories {
        mavenCentral()
    }

    group = "club.minnced"
    version = "0.2.10"
}
