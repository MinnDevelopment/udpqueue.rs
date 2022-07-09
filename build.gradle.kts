plugins {
    `java-library`
    `maven-publish`
}

allprojects {
    repositories {
        mavenCentral()
        maven("https://jcenter.bintray.com/")
    }

    group = "dev.minn"
    version = "0.1.0"

    fun getPlatform(triplet: String) = when {
        triplet.startsWith("x86_64")  && "linux"   in triplet -> "linux-x86-64"
        triplet.startsWith("x86")     && "linux"   in triplet -> "linux-x86"
        triplet.startsWith("aarch64") && "linux"   in triplet -> "linux-aarch64"
        triplet.startsWith("arm")     && "linux"   in triplet -> "linux-arm"

        triplet.startsWith("x86_64")  && "windows" in triplet -> "win-x86-64"
        triplet.startsWith("x86")     && "windows" in triplet -> "win-x86"
        triplet.startsWith("aarch64") && "windows" in triplet -> "win-aarch64"
        triplet.startsWith("arm")     && "windows" in triplet -> "win-arm"

        triplet.startsWith("x86_64")  && "darwin"  in triplet -> "darwin"
        triplet.startsWith("x86")     && "darwin"  in triplet -> "darwin"
        triplet.startsWith("aarch64") && "darwin"  in triplet -> "darwin"
        triplet.startsWith("arm")     && "darwin"  in triplet -> "darwin"

        triplet.isEmpty() -> "linux-x86-64" // TODO: find current OS instead
        else -> throw IllegalArgumentException("Unknown platform: $triplet")
    }

    ext["platform"] = getPlatform(project.properties["target"] as? String ?: "")
}
