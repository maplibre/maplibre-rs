package org.maplibre_rs;

public class MapLibreRs {
    public static native void android_main();

    static {
        System.loadLibrary("maplibre_android");
    }
}
