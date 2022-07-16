plugins {
    `java-library`
    `maven-publish`
    signing
//    id("io.github.gradle-nexus.publish-plugin")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.sedmelluq:lava-common:1.1.2")
    implementation("org.slf4j:slf4j-api:1.7.25")
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

if (ext.has("signingKeyId")) {
    signing {
        sign(publishing.publications["Release"])
        if (ext.has("signingKey")) {
            useInMemoryPgpKeys(
                ext["signingKeyId"].toString(),
                ext["signingKey"].toString(),
                null
            )
        }
    }
}
