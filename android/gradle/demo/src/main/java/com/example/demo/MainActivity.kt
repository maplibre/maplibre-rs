package com.example.demo

import android.os.Bundle
import android.util.Log
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import androidx.appcompat.app.AppCompatActivity
import org.maplibre_rs.MapLibreRs

// Currently not used. Instead the NativeActivity referenced in AndroidManifest.xml is used.

class MainActivity : AppCompatActivity() {

    var mSurfaceView1: SurfaceView? = null
    var mSurfaceHolder1: SurfaceHolder? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        mSurfaceView1 = findViewById<View>(R.id.surfaceView1) as SurfaceView
        mSurfaceHolder1 = mSurfaceView1!!.getHolder()

        mSurfaceHolder1!!.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(p0: SurfaceHolder) {
                Log.v("TAG", "surfaceCreated")
                MapLibreRs.android_main(p0.surface)
            }
            
            override fun surfaceChanged(p0: SurfaceHolder, p1: Int, p2: Int, p3: Int) {
                Log.v(
                    "TAG", "surfaceChanged format=" + p1 + ", width=" + p2 + ", height="
                            + p3
                )
            }

            override fun surfaceDestroyed(p0: SurfaceHolder) {
                Log.v("TAG", "surfaceDestroyed")
            }
        })
    }
}
