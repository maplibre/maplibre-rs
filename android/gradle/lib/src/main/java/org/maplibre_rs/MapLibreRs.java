package org.maplibre_rs;

import android.view.Surface;

public class MapLibreRs {
    public static native void android_main(Surface surface);

    static {
        System.loadLibrary("maplibre_android");
    }
}
