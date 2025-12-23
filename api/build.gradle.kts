plugins {
    `publishing-environment`
}

publishingEnvironment {
    moduleName = "udpqueue-api"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.slf4j:slf4j-api:2.0.17")
    implementation("dev.arbjerg:lava-common:1.5.6")
    compileOnly("net.dv8tion:JDA:6.2.0")
}

val javadoc: Javadoc by tasks

javadoc.isFailOnError = false
