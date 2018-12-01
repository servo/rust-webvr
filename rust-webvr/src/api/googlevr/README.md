
This module includes an Android Archive (`aar/GVRService.aar`). If you're modifying any of the Java/Android files under `gradle/` this file needs to be regenerated.

This can be done by:

 - Opening `gradle/` as an Android Studio project
 - Setting the build variant to `release` (View -> Tool Windows -> Build Variants)
 - Running a build (Make Project, Ctrl-F9)
 - Copying `gradle/build/outputs/aar/GVRService.aar` to `aar/GVRService.aar` and checking it in

The build can also be run standalone via `./gradleW assembleRelease`