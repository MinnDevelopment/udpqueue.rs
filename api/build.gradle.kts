plugins {
    `java-library`
    `maven-publish`
}

repositories {
  mavenCentral()
}

dependencies {
  implementation("com.sedmelluq:lava-common:1.1.0")
  implementation("org.slf4j:slf4j-api:1.7.25")
}

publishing.publications {
    create<MavenPublication>("Maven") {
        from(components["java"])

        artifactId = "udpqueue-api"
    }
}