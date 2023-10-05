package org.maplibre_rs;

import android.os.Environment;

public class MapLibreRs {
    public static void start() {
        android_main();
    }
    
    public static native void android_main();

    static {
        System.loadLibrary("maplibre_android");
    }
}
