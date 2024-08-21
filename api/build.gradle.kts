plugins {
    `java-library`
    `maven-publish`
    signing
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.slf4j:slf4j-api:1.7.25")
    implementation("dev.arbjerg:lava-common:1.5.4")
}

val javadoc: Javadoc by tasks

javadoc.isFailOnError = false

val javadocJar = tasks.register<Jar>("javadocJar") {
    group = "build"
    dependsOn(javadoc)
    from(javadoc.destinationDir)
    archiveClassifier.set("javadoc")
}

val sourcesJar = tasks.register<Jar>("sourcesJar") {
    group = "build"
    from(sourceSets["main"].java)
    archiveClassifier.set("sources")
}

tasks.withType<Jar> {
    archiveBaseName.set("udpqueue-api")
}

publishing.publications {
    create<MavenPublication>("Release") {
        from(components["java"])

        groupId = group.toString()
        artifactId = "udpqueue-api"
        version = version.toString()

        artifact(javadocJar)
        artifact(sourcesJar)

        pom.apply(ext["generatePom"] as MavenPom.() -> Unit)
        pom.name.set(artifactId)
    }
}

val signingKey: String? by project

if (signingKey != null) {
    signing {
        useInMemoryPgpKeys(signingKey, null)
        sign(*publishing.publications.toTypedArray())
    }
}