import org.gradle.kotlin.dsl.withType
import org.jreleaser.gradle.plugin.JReleaserExtension
import org.jreleaser.gradle.plugin.tasks.AbstractJReleaserTask
import org.jreleaser.model.Active
import udpqueue.gradle.generatePom
import udpqueue.gradle.stagingDirectory

plugins {
    `java-library`
    `maven-publish`

    id("org.jreleaser")
}


interface PublishingEnvironmentExtension {
    val moduleName: Property<String>
}

val publishingEnvironment = project.extensions.create<PublishingEnvironmentExtension>("publishingEnvironment")
val sourceSets = the<SourceSetContainer>()

val javadoc: Javadoc by tasks
val javadocJar by tasks.registering(Jar::class) {
    group = "build"
    dependsOn(javadoc)
    from(javadoc.destinationDir)
    archiveClassifier.set("javadoc")
}

val sourcesJar by tasks.registering(Jar::class) {
    group = "build"
    from(sourceSets["main"].java)
    archiveClassifier.set("sources")
}

configure<JavaPluginExtension> {
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8

    afterEvaluate {
        base.archivesName = publishingEnvironment.moduleName
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.encoding = "UTF-8"
    options.release = 8
}

configure<PublishingExtension> {
    publications {
        register<MavenPublication>("Release") {
            from(components["java"])

            afterEvaluate {
                artifactId = publishingEnvironment.moduleName.get()
                pom.name.set(artifactId)
            }

            groupId = group.toString()
            version = version.toString()

            artifact(tasks.named("javadocJar"))
            artifact(tasks.named("sourcesJar"))

            pom.generatePom()
        }
    }

    repositories.maven {
        url = stagingDirectory.asFile.toURI()
    }
}

configure<JReleaserExtension> {
    gitRootSearch = true

    release {
        github {
            enabled = false
        }
    }

    signing {
        active = Active.RELEASE
        armored = true
    }

    deploy {
        maven {
            mavenCentral {
                register("sonatype") {
                    active = Active.RELEASE
                    url = "https://central.sonatype.com/api/v1/publisher"
                    stagingRepository(stagingDirectory.asFile.relativeTo(projectDir).path)
                }
            }
        }
    }
}

tasks.withType<AbstractJReleaserTask>().configureEach {
    mustRunAfter(tasks.named("publish"))
}
