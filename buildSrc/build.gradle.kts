import org.gradle.kotlin.dsl.withType
import org.jetbrains.kotlin.gradle.dsl.jvm.JvmTargetValidationMode
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    `kotlin-dsl`
    id("java-gradle-plugin")
}

repositories {
    mavenCentral()
    gradlePluginPortal()
}

dependencies {
    implementation(gradleApi())
    implementation("org.jreleaser:jreleaser-gradle-plugin:1.20.0")
}

kotlin {
    jvmToolchain(17)
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

tasks.withType(KotlinCompile::class).configureEach {
    jvmTargetValidationMode.set(JvmTargetValidationMode.IGNORE)
}