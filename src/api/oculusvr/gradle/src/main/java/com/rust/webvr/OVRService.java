package com.rust.webvr;

import android.app.Activity;
import android.app.Application;
import android.os.Bundle;


class OVRService  implements Application.ActivityLifecycleCallbacks {
    private Activity mActivity;
    private long mPtr = 0; // Native Rustlang struct pointer

    private static native void nativeOnPause(long ptr);
    private static native void nativeOnResume(long ptr);

    void init(final Activity activity, long ptr) {
        mActivity = activity;
        mPtr = ptr;
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                activity.getApplication().registerActivityLifecycleCallbacks(OVRService.this);
            }
        });
    }

    // Called from JNI
    public static Object create(Activity activity, long ptr) {
        OVRService service = new OVRService();
        service.init(activity, ptr);
        return service;
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
}
