package com.rust.webvr;

import android.app.Activity;
import android.app.Application;
import android.os.Bundle;
import android.util.Log;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.widget.FrameLayout;
import android.widget.FrameLayout.LayoutParams;

class OVRService  implements Application.ActivityLifecycleCallbacks, SurfaceHolder.Callback {
    private Activity mActivity;
    private SurfaceView mSurfaceView;
    private long mPtr = 0; // Native Rustlang struct pointer
    private boolean mPresenting = false;

    private static native void nativeOnPause(long ptr);
    private static native void nativeOnResume(long ptr);
    private static native void nativeOnSurfaceChanged(long ptr, Surface surface);
    private static native void nativeOnSurfaceDestroyed(long ptr);

    void init(final Activity activity, long ptr) {
        mActivity = activity;
        mPtr = ptr;

        Runnable initOvr = new Runnable() {
            @Override
            public void run() {
                mSurfaceView = new SurfaceView(activity);
                mSurfaceView.getHolder().addCallback(OVRService.this);
                // Enabling setZOrderOnTop is very important! If not enabled a simple swap_buffers
                // on the window will make the SurfaceView invisible
                mSurfaceView.setZOrderOnTop(true);
                activity.getApplication().registerActivityLifecycleCallbacks(OVRService.this);

                // Wait until completed
                synchronized(this) {
                    this.notify();
                }
            }
        };

        synchronized (initOvr) {
            activity.runOnUiThread(initOvr);
            try {
                initOvr.wait();
            }
            catch (Exception ex) {
                Log.e("rust-webvr", Log.getStackTraceString(ex));
            }
        }
    }

    // Called from JNI
    public static Object create(Activity activity, long ptr) {
        OVRService service = new OVRService();
        service.init(activity, ptr);
        return service;
    }

    // Called from Native
    public void startPresent() {
        mActivity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (mPresenting) {
                    return;
                }
                // Show SurfaceView
                FrameLayout rootLayout = (FrameLayout) mActivity.findViewById(android.R.id.content);
                rootLayout.addView(mSurfaceView, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));

                mPresenting = true;
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
                // Hide SurfaceView
                FrameLayout rootLayout = (FrameLayout)mActivity.findViewById(android.R.id.content);
                rootLayout.removeView(mSurfaceView);
            }
        });
    }

    // ActivityLifecycleCallbacks
    @Override
    public void onActivityCreated(Activity activity, Bundle savedInstanceState) {

    }

    @Override
    public void onActivityStarted(Activity activity) {

    }

    @Override
    public void onActivityResumed(Activity activity) {
        if (activity != mActivity) {
            return;
        }
        nativeOnResume(mPtr);
    }

    @Override
    public void onActivityPaused(Activity activity) {
        if (activity != mActivity) {
            return;
        }
        nativeOnPause(mPtr);
    }

    @Override
    public void onActivityStopped(Activity activity) {

    }

    @Override
    public void onActivitySaveInstanceState(Activity activity, Bundle outState) {

    }

    @Override
    public void onActivityDestroyed(Activity activity) {
        if (mActivity == activity) {
            activity.getApplication().unregisterActivityLifecycleCallbacks(this);
            mActivity = null; // Don't leak activity
        }
    }

    // SurfaceView Callbacks
    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        // No Op
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        nativeOnSurfaceChanged(mPtr, holder.getSurface());
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        nativeOnSurfaceDestroyed(mPtr);
    }
}
