package com.example.demo

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import org.maplibre_rs.MapLibreRs

// Currently not used. Instead the NativeActivity referenced in AndroidManifest.xml is used.

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        MapLibreRs.start()
    }
}
