import java.time.Duration

plugins {
    `java-library`
    `maven-publish`
    signing
    id("io.github.gradle-nexus.publish-plugin") version "1.1.0"
}

fun getOption(name: String) = System.getenv(name) ?: project.findProperty(name)?.toString()

if (listOf("OSSRH_USER", "OSSRH_PASSWORD", "STAGING_PROFILE_ID").all { getOption(it) != null }) {
    apply(plugin = "io.github.gradle-nexus.publish-plugin")

    nexusPublishing {
        repositories.sonatype {
            username.set(getOption("OSSRH_USER"))
            password.set(getOption("OSSRH_PASSWORD"))
            stagingProfileId.set(getOption("STAGING_PROFILE_ID"))
        }

        // Sonatype is very slow :)
        connectTimeout.set(Duration.ofMinutes(1))
        clientTimeout.set(Duration.ofMinutes(10))

        transitionCheckOptions {
            maxRetries.set(100)
            delayBetween.set(Duration.ofSeconds(5))
        }
    }
}

subprojects {
    repositories {
        mavenCentral()
        maven("https://jcenter.bintray.com/")
    }

    apply(plugin="java")

    configure<JavaPluginExtension> {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    group = "club.minnced"
    version = "0.1.1-rc"

    fun getPlatform(triplet: String) = when {
        triplet.startsWith("x86_64")  && "linux"   in triplet -> "linux-x86-64"
        triplet.startsWith("i686")    && "linux"   in triplet -> "linux-x86"
        triplet.startsWith("aarch64") && "linux"   in triplet -> "linux-aarch64"
        triplet.startsWith("arm")     && "linux"   in triplet -> "linux-arm"

        triplet.startsWith("x86_64")  && "windows" in triplet -> "win-x86-64"
        triplet.startsWith("i686")    && "windows" in triplet -> "win-x86"
        triplet.startsWith("aarch64") && "windows" in triplet -> "win-aarch64"
        triplet.startsWith("arm")     && "windows" in triplet -> "win-arm"

        triplet.startsWith("x86_64")  && "darwin"  in triplet -> "darwin"
        triplet.startsWith("i686")    && "darwin"  in triplet -> "darwin"
        triplet.startsWith("aarch64") && "darwin"  in triplet -> "darwin"
        triplet.startsWith("arm")     && "darwin"  in triplet -> "darwin"

        else -> throw IllegalArgumentException("Unknown platform: $triplet")
    }

    // Testing: "x86_64-unknown-linux-gnu"
    ext["target"] = "x86_64-unknown-linux-gnu" // project.property("target") as? String ?: throw AssertionError("Invalid target")
    ext["platform"] = getPlatform(ext["target"].toString())
    ext["signingKey"] = getOption("GPG_KEY")
    ext["signingKeyId"] = getOption("GPG_KEYID")

    val generatePom: MavenPom.() -> Unit = {
        packaging = "jar"
        description.set("Rust implementation of the JDA-NAS interface")
        url.set("https://github.com/MinnDevelopment/udpqueue.rs")
        scm {
            url.set("https://github.com/MinnDevelopment/udpqueue.rs")
            connection.set("scm:git:git://github.com/MinnDevelopment/udpqueue.rs")
            developerConnection.set("scm:git:ssh:git@github.com:MinnDevelopment/udpqueue.rs")
        }
        licenses {
            license {
                name.set("The Apache Software License, Version 2.0")
                url.set("https://www.apache.org/licenses/LICENSE-2.0.txt")
                distribution.set("repo")
            }
        }
        developers {
            developer {
                id.set("Minn")
                name.set("Florian Spieß")
                email.set("business@minn.dev")
            }
        }
    }

    ext["generatePom"] = generatePom

    val rebuild = tasks.create("rebuild") {
        group = "build"
        afterEvaluate {
            dependsOn(tasks["build"], tasks["clean"])
            tasks["build"].dependsOn(tasks.withType<Jar>())
            tasks.forEach {
                if (it.name != "clean")
                    mustRunAfter(tasks["clean"])
            }
        }
    }

    val publishingTasks = tasks.withType<PublishToMavenRepository> {
        enabled = "ossrhUser" in properties
        mustRunAfter(rebuild)
        dependsOn(rebuild)
    }

//    tasks.create("release") {
//        group = "publishing"
//        dependsOn(publishingTasks)
//        afterEvaluate {
//            // Collect all the publishing task which upload the archives to nexus staging
//            val closeAndReleaseSonatypeStagingRepository: Task by tasks
//
//            // Make sure the close and release happens after uploading
//            dependsOn(closeAndReleaseSonatypeStagingRepository)
//            closeAndReleaseSonatypeStagingRepository.mustRunAfter(publishingTasks)
//        }
//    }
}
