import java.net.URL
import javax.net.ssl.HttpsURLConnection

plugins {
    `java-library`
    signing
    `maven-publish`
}

dependencies {
    // Explicit dependency to avoid having to republish api each time
    api("club.minnced:udpqueue-api:0.1.1")
}

val processResources: Copy by tasks
val target = ext["target"]?.toString() ?: ""
val platform = ext["platform"] as String
val artifactName = "udpqueue-native-$platform"

// This checks if the version already exists on maven central, and skips if a successful response is returned.
val shouldPublish by lazy {
    val conn = URL("https://repo1.maven.org/maven2/club/minnced/$artifactName/$version/").openConnection() as HttpsURLConnection
    conn.requestMethod = "GET"
    conn.connect()

    conn.responseCode > 400
}

tasks.withType<Jar> {
    archiveBaseName.set(artifactName)
}

tasks.create<Copy>("moveResources") {
    group = "build"

    from("target/$target/release/")

    include {
        it.name == "release" || it.name.endsWith(".so") || it.name.endsWith(".dll") || it.name.endsWith(".dylib")
    }

    into("src/main/resources/natives/$platform")

    processResources.dependsOn(this)
}

tasks.create<Delete>("cleanNatives") {
    group = "build"
    delete(fileTree("src/main/resources/natives"))
    tasks["clean"].dependsOn(this)
}

processResources.include {
    it.isDirectory || it.file.parentFile.name == platform
}


publishing.publications {
    create<MavenPublication>("Release") {
        from(components["java"])

        groupId = group.toString()
        artifactId = artifactName
        version = version.toString()

        pom.apply(ext["generatePom"] as MavenPom.() -> Unit)
        pom.name.set(artifactName)
    }
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