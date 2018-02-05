package com.rust.webvr;
import android.graphics.Color;
import android.opengl.GLES20;
import android.os.Bundle;
import android.os.SystemClock;
import android.util.AttributeSet;
import android.util.SparseArray;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.ViewGroup;
import android.view.WindowManager;
import android.view.animation.Animation;
import android.view.animation.LinearInterpolator;
import android.view.animation.RotateAnimation;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.FrameLayout;
import android.widget.ImageButton;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.lang.System;
import java.util.ArrayList;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

public class MainActivity extends android.app.NativeActivity {
    private static final String LOGTAG = "WebVRExample";
    private FrameLayout mContentView;
    private ArrayList<SurfaceTextureRenderer> mRenderers = new ArrayList<>();
    private SparseArray<View> mViews = new SparseArray<>();

    static {
        //System.loadLibrary("gvr");
        //System.loadLibrary("gvr_audio");
        System.loadLibrary("vrapi");
        System.loadLibrary("webvr");
    }

    @Override
    public void onCreate(Bundle savedInstanceState) {
        try {
            extractAssets();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        super.onCreate(savedInstanceState);
        getWindow().takeSurface(null);

        mContentView = new FrameLayout(this);
        mContentView.setLayoutParams(new FrameLayout.LayoutParams(FrameLayout.LayoutParams.MATCH_PARENT,
                                                            FrameLayout.LayoutParams.MATCH_PARENT));
        SurfaceView nativeSurface = new SurfaceView(this);
        nativeSurface.getHolder().addCallback(this);
        mContentView.addView(nativeSurface, new FrameLayout.LayoutParams(FrameLayout.LayoutParams.MATCH_PARENT, FrameLayout.LayoutParams.MATCH_PARENT));
        setContentView(mContentView);

        keepScreenOn();
        addFullScreenListener();
    }

    @Override
    protected void onResume() {
        setFullScreen();
        super.onResume();
    }

    // keep the device's screen turned on and bright.
    private void keepScreenOn() {
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
    }

    // Dim toolbar and make the view fullscreen
    private void setFullScreen() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                | View.SYSTEM_UI_FLAG_FULLSCREEN
                | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY);
    }

    // This is here to make the toolbar autohide after 2 seconds of being touched
    private void addFullScreenListener() {
        View decorView = getWindow().getDecorView();
        decorView.setOnSystemUiVisibilityChangeListener(
                new View.OnSystemUiVisibilityChangeListener() {
                    public void onSystemUiVisibilityChange(int visibility) {
                        if ((visibility & View.SYSTEM_UI_FLAG_FULLSCREEN) == 0) {
                            setFullScreen();
                        }
                    }
                });
    }

    private void extractAssets() throws IOException {
        String path = getExternalFilesDir(null) + "/res";

        ZipFile zipFile = null;
        File targetDir = new File(path);
        try {
            zipFile = new ZipFile(this.getApplicationInfo().sourceDir);
            for (Enumeration<? extends ZipEntry> e = zipFile.entries(); e.hasMoreElements(); ) {
                ZipEntry entry = e.nextElement();
                if (entry.isDirectory() || !entry.getName().startsWith("assets/")) {
                    continue;
                }
                File targetFile = new File(targetDir, entry.getName().substring("assets/".length()));
                targetFile.getParentFile().mkdirs();
                byte[] tempBuffer = new byte[(int)entry.getSize()];
                BufferedInputStream is = null;
                FileOutputStream os = null;
                try {
                    is = new BufferedInputStream(zipFile.getInputStream(entry));
                    os = new FileOutputStream(targetFile);
                    is.read(tempBuffer);
                    os.write(tempBuffer);
                } finally {
                    if (is != null) is.close();
                    if (os != null) os.close();
                }
            }
        } finally {
            if (zipFile != null) zipFile.close();
        }
    }

    private int createAndroidView(final int width, final int height)
    {
        SurfaceTextureRenderer renderer = new SurfaceTextureRenderer();
        renderer.initialize(width, height);
        mRenderers.add(renderer);

        this.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                View root = getLayoutInflater().inflate(R.layout.tab, null);
                GLFrameLayout view = (GLFrameLayout)root.findViewById(R.id.container);
                //GLWebView view = new GLWebView(MainActivity.this);
                WebView webView = (WebView)root.findViewById(R.id.webview);
                view.setRenderer(renderer);
                mContentView.addView(root, new FrameLayout.LayoutParams(width, height));
                webView.getSettings().setJavaScriptEnabled(true);
                webView.setWebViewClient(new WebViewClient(){
                    public boolean shouldOverrideUrlLoading(WebView view, String url) {
                        return false;
                    }
                });
                webView.loadUrl("https://www.reddit.com");
                mViews.append(renderer.textureId(), root);

                RotateAnimation rotate = new RotateAnimation(0, 180, Animation.RELATIVE_TO_SELF, 0.5f, Animation.RELATIVE_TO_SELF, 0.5f);
                rotate.setDuration(1000);
                rotate.setRepeatCount(1000);
                rotate.setInterpolator(new LinearInterpolator());

                ImageButton image= (ImageButton) findViewById(R.id.reloadButton);

                image.startAnimation(rotate);
            }
        });

        return renderer.textureId();
    }

    private void updateSurfaceTextures()
    {
        for(int i = 0; i < mViews.size(); i++) {
            View view = mViews.valueAt(i);
            view.postInvalidate();
        }
        for (SurfaceTextureRenderer renderer: mRenderers) {
            renderer.updateTexture();
        }
    }

    private View mTouchingView = null;

    private void mapInput(int id, int x, int y, boolean pressed) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                handleInput(id, x, y, pressed);
            }
        });
    }

    private void handleInput(int id, int x, int y, boolean pressed) {
        View target = null;
        int action = 0;

        if (mTouchingView != null && !pressed) {
            // Handle touchend
            target = mTouchingView;
            action = MotionEvent.ACTION_UP;
            mTouchingView = null;
        } else if (mTouchingView != null) {
            // Handle touchmove
            target = mTouchingView;
            action = MotionEvent.ACTION_MOVE;
        } else if (pressed) {
            // Handle touch start
            mTouchingView = mViews.get(id);
            target = mTouchingView;
            action = MotionEvent.ACTION_DOWN;
        }

        if (target != null) {
            long ms = SystemClock.uptimeMillis();
            MotionEvent e = MotionEvent.obtain(ms, ms, action, x, y, 0);
            target.dispatchTouchEvent(e);
            e.recycle();
        }
    }

}
