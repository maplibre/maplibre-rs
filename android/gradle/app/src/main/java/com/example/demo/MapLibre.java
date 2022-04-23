package com.example.demo;

public class MapLibre {
    public static native void android_main();

    static {
        System.loadLibrary("maplibre_android");
    }
}
