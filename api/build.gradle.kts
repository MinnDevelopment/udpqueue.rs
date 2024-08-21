import java.net.URL
import javax.net.ssl.HttpsURLConnection

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


val shouldPublish by lazy {
    val conn = URL("https://repo1.maven.org/maven2/club/minnced/udpqueue-api/$version/").openConnection() as HttpsURLConnection
    conn.requestMethod = "GET"
    conn.connect()

    conn.responseCode > 400
}

val signingKey: String? by project
val signingPassword: String? by project

if (signingKey != null) {
    signing {
        useInMemoryPgpKeys(signingKey, signingPassword ?: "")
        val publications = publishing.publications.toTypedArray()
        sign(*publications)
    }
} else {
    println("Could not find signingKey")
}

// Only run publishing tasks if the version doesn't already exist

tasks.withType<PublishToMavenRepository> {
    enabled = enabled && shouldPublish
}