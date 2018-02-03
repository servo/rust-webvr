package com.rust.webvr;

import android.content.Context;
import android.graphics.Canvas;
import android.util.AttributeSet;
import android.webkit.WebView;

public class GLWebView extends WebView {
    private SurfaceTextureRenderer mRenderer;

    public GLWebView(Context context) {
        super(context);
    }

    public GLWebView(Context context, AttributeSet attrs) {
        super(context, attrs);
    }

    public GLWebView(Context context, AttributeSet attrs, int defStyle) {
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

    public void setRenderer(SurfaceTextureRenderer viewTOGLRenderer){
        mRenderer = viewTOGLRenderer;
    }
}
