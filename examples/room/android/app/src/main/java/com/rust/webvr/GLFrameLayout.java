package com.rust.webvr;

import android.annotation.TargetApi;
import android.content.Context;
import android.graphics.Canvas;
import android.os.Build;
import android.util.AttributeSet;
import android.widget.FrameLayout;

import com.rust.webvr.SurfaceTextureRenderer;


public class GLFrameLayout extends FrameLayout {
    private SurfaceTextureRenderer mRenderer;

    // default constructors
    public GLFrameLayout(Context context) {
        super(context);
    }

    public GLFrameLayout(Context context, AttributeSet attrs) {
        super(context, attrs);
    }

    public GLFrameLayout(Context context, AttributeSet attrs, int defStyle) {
        super(context, attrs, defStyle);
    }

    @Override
    public void draw( Canvas canvas ) {
        if (mRenderer == null) {
            super.draw(canvas);
            return;
        }
        Canvas textureCanvas = mRenderer.drawBegin();
        if(textureCanvas != null) {
            // set the proper scale and translations
            float xScale = textureCanvas.getWidth() / (float)canvas.getWidth();
            textureCanvas.scale(xScale, xScale);
            //textureCanvas.translate(-getScrollX(), -getScrollY());
            //draw the view to SurfaceTexture
            super.draw(textureCanvas);
        }
        mRenderer.drawEnd();
    }

    public void setRenderer(SurfaceTextureRenderer viewTOGLRenderer) {
        mRenderer = viewTOGLRenderer;
        setWillNotDraw(mRenderer == null);
    }
}
