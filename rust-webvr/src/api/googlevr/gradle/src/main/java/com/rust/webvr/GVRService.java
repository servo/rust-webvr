package com.rust.webvr;

import android.app.Activity;
import android.app.Application;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.FrameLayout.LayoutParams;

import com.google.vr.ndk.base.AndroidCompat;
import com.google.vr.ndk.base.GvrLayout;

class GVRService  implements Application.ActivityLifecycleCallbacks {
    private Activity mActivity;
    private GvrLayout gvrLayout;
    private long mPtr = 0; // Native Rustlang struct pointer
    private boolean mPresenting = false;
    private boolean mPaused = false;
    private boolean mGvrResumed = false;

    private static native void nativeOnPause(long ptr);
    private static native void nativeOnResume(long ptr);

    void init(final Activity activity, long ptr) {
        mActivity = activity;
        mPtr = ptr;

        Runnable initGvr = new Runnable() {
            @Override
            public void run() {
                gvrLayout = new GvrLayout(activity);
                // Decouple the app framerate from the display framerate
                if (gvrLayout.setAsyncReprojectionEnabled(true)) {
                    // Android N hint to tune apps for a predictable,
                    // consistent level of device performance over long periods of time.
                    // The system automatically disables this mode when the window
                    // is no longer in focus.
                    AndroidCompat.setSustainedPerformanceMode(activity, true);
                }
                gvrLayout.setPresentationView(new View(activity));

                activity.getApplication().registerActivityLifecycleCallbacks(GVRService.this);

                // Wait until completed
                synchronized(this) {
                    this.notify();
                }
            }
        };

        synchronized (initGvr) {
            activity.runOnUiThread(initGvr);
            try {
                initGvr.wait();
            }
            catch (Exception ex) {
                Log.e("rust-webvr", Log.getStackTraceString(ex));
            }
        }
    }

    // Called from Native
    public long getNativeContext() {
        return gvrLayout.getGvrApi().getNativeGvrContext();
    }

    private void start() {
        if (mPresenting) {
            return;
        }

        mActivity.setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE);
        mActivity.getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN, WindowManager.LayoutParams.FLAG_FULLSCREEN);
        if (!AndroidCompat.setVrModeEnabled(mActivity, true)) {
          Log.w("rust-webvr", "setVrModeEnabled failed");
        }

        // Show GvrLayout
        FrameLayout rootLayout = (FrameLayout)mActivity.findViewById(android.R.id.content);
        rootLayout.addView(gvrLayout, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));

        if (!mGvrResumed) {
            gvrLayout.onResume();
            mGvrResumed = true;
        }
        mPresenting = true;
    }


    // Called from Native
    public void startPresent() {
        mActivity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                start();
            }
        });
    }

    public void stopPresent() {
        mActivity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (!mPresenting) {
                    return;
                }
                mPresenting = false;
                // Hide GvrLayout
                FrameLayout rootLayout = (FrameLayout)mActivity.findViewById(android.R.id.content);
                rootLayout.removeView(gvrLayout);

                AndroidCompat.setVrModeEnabled(mActivity, false);
                mActivity.setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_USER);
                mActivity.getWindow().clearFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN);
            }
        });
    }

    // Called from JNI
    public static Object create(Activity activity, long ptr) {
        GVRService service = new GVRService();
        service.init(activity, ptr);
        return service;
    }

    // ActivityLifecycleCallbacks
    @Override
    public void onActivityCreated(Activity activity, Bundle savedInstanceState) {

    }

    @Override
    public void onActivityStarted(Activity activity) {
        if (activity != mActivity) {
            return;
        }
        if (mPaused && gvrLayout != null && !mGvrResumed) {
            gvrLayout.onResume();
            mGvrResumed = true;
            mPaused = false;
            nativeOnResume(mPtr);
        }
    }

    @Override
    public void onActivityResumed(Activity activity) {

    }

    @Override
    public void onActivityPaused(Activity activity) {

    }

    @Override
    public void onActivityStopped(Activity activity) {
        if (activity != mActivity) {
            return;
        }

        if (mPresenting && gvrLayout != null && mGvrResumed) {
            gvrLayout.onPause();
            mGvrResumed = false;
            mPaused = true;
            nativeOnPause(mPtr);
        }
    }

    @Override
    public void onActivitySaveInstanceState(Activity activity, Bundle outState) {

    }

    @Override
    public void onActivityDestroyed(Activity activity) {
        if (mActivity == activity) {
            if (gvrLayout != null) {
                gvrLayout.shutdown();
                gvrLayout = null;
            }
            activity.getApplication().unregisterActivityLifecycleCallbacks(this);
            mActivity = null; // Don't leak activity
        }
    }
}
