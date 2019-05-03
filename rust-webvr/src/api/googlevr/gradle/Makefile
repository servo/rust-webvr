SOURCES = \
  src/main/java/com/rust/webvr/GVRService.java \
  src/main/AndroidManifest.xml \
  build.gradle \
  settings.gradle \
  gradle/wrapper/gradle-wrapper.properties \
  gradle/wrapper/gradle-wrapper.jar

OUTPUT_AAR = build/outputs/aar/GVRService-release.aar
FINAL_AAR = ../aar/GVRService.aar

all: $(FINAL_AAR)

$(OUTPUT_AAR): $(SOURCES)	
	./gradlew assembleRelease && touch $(OUTPUT_AAR)

$(FINAL_AAR): $(OUTPUT_AAR)
	cp $(OUTPUT_AAR) $(FINAL_AAR)
