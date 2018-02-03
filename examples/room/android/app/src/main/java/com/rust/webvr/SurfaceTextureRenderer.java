package com.rust.webvr;

import android.graphics.Canvas;
import android.graphics.SurfaceTexture;
import android.opengl.GLES11Ext;
import android.opengl.GLES20;
import android.view.Surface;


public class SurfaceTextureRenderer {
    private int mTextureWidth;
    private int mTextureHeight;
    private int mTextureId;
    private SurfaceTexture mSurfaceTexture;
    private Surface mSurface;
    private Canvas mSurfaceCanvas;

    public void initialize(int width, int height) {
        release();
        mTextureWidth = width;
        mTextureHeight = height;

        mTextureId = genExternalTexture();
        mSurfaceTexture = new SurfaceTexture(mTextureId);
        mSurfaceTexture.setDefaultBufferSize(mTextureWidth, mTextureHeight);
        mSurface = new Surface(mSurfaceTexture);
    }

    public void release() {
        if(mSurface != null){
            mSurface.release();
        }
        if(mSurfaceTexture != null){
            mSurfaceTexture.release();
        }
        mSurface = null;
        mSurfaceTexture = null;
    }

    public Canvas drawBegin() {
        mSurfaceCanvas = null;
        if (mSurface != null) {
            try {
                mSurfaceCanvas = mSurface.lockCanvas(null);
            }
            catch (Exception e){
                e.printStackTrace();
            }
        }
        return mSurfaceCanvas;
    }

    public void drawEnd() {
        if(mSurfaceCanvas != null) {
            mSurface.unlockCanvasAndPost(mSurfaceCanvas);
        }
        mSurfaceCanvas = null;
    }

    public void updateTexture() {
        if (mSurfaceTexture != null) {
            mSurfaceTexture.updateTexImage();
        }
    }

    public int width() {
        return mTextureWidth;
    }

    public int height() {
        return mTextureHeight;
    }

    public int textureId() {
        return mTextureId;
    }

    private int genExternalTexture(){
        int[] textures = new int[1];
        GLES20.glGenTextures(1, textures, 0);
        checkGlError("Generate external texture");

        GLES20.glBindTexture(GLES11Ext.GL_TEXTURE_EXTERNAL_OES, textures[0]);
        checkGlError("Bind external texture");

        GLES20.glTexParameterf(GLES11Ext.GL_TEXTURE_EXTERNAL_OES, GLES20.GL_TEXTURE_MIN_FILTER, GLES20.GL_LINEAR);
        GLES20.glTexParameterf(GLES11Ext.GL_TEXTURE_EXTERNAL_OES, GLES20.GL_TEXTURE_MAG_FILTER, GLES20.GL_LINEAR);
        GLES20.glTexParameteri(GLES11Ext.GL_TEXTURE_EXTERNAL_OES, GLES20.GL_TEXTURE_WRAP_S, GLES20.GL_CLAMP_TO_EDGE);
        GLES20.glTexParameteri(GLES11Ext.GL_TEXTURE_EXTERNAL_OES, GLES20.GL_TEXTURE_WRAP_T, GLES20.GL_CLAMP_TO_EDGE);

        return textures[0];
    }

    public void checkGlError(String action) {
        int error = GLES20.glGetError();
        if (error != GLES20.GL_NO_ERROR) {
            throw new RuntimeException("OpenGL Error in (" + action + "): " + error);
        }
    }

}
