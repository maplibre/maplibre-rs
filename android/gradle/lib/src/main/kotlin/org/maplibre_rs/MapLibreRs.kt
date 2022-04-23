package org.maplibre_rs

class MapLibreRs {
    companion object {
        @JvmStatic fun android_main() {}
        
        init {
            System.loadLibrary("maplibre_android")
        }
    }
}
