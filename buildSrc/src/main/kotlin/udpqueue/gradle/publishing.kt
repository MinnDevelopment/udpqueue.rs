package udpqueue.gradle

import org.gradle.api.Project
import org.gradle.api.file.Directory
import org.gradle.api.publish.maven.MavenPom

fun MavenPom.generatePom() {
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

val Project.stagingDirectory: Directory get() = layout.buildDirectory.dir("staging-deploy").get()
