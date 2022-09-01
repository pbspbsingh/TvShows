package com.pbs.tvshowsapp

import android.annotation.SuppressLint
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Bundle
import android.view.View
import android.webkit.WebChromeClient
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.FrameLayout
import androidx.appcompat.app.AppCompatActivity
import com.pbs.tvshows.server.TvShowsServer


class MainActivity : AppCompatActivity() {

  private val webview by lazy<WebView>(LazyThreadSafetyMode.NONE) {
    findViewById(R.id.webview)
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    setContentView(R.layout.activity_main)
    initWebView()

    if (savedInstanceState == null) {
      TvShowsServer.startServerInBackground(cacheDir.canonicalPath + "/serverCache", PORT)
      webview.loadUrl("http://127.0.0.1:$PORT")
    }
  }

  override fun onSaveInstanceState(outState: Bundle) {
    super.onSaveInstanceState(outState)
    webview.saveState(outState)
  }

  override fun onRestoreInstanceState(savedInstanceState: Bundle) {
    super.onRestoreInstanceState(savedInstanceState)
    webview.restoreState(savedInstanceState)
  }

  @SuppressLint("SetJavaScriptEnabled")
  private fun initWebView() {
    with(webview) {
      webViewClient = WebViewClient()
      webChromeClient = MyChrome()

      settings.javaScriptEnabled = true
      settings.allowFileAccess = true
      settings.setAppCacheEnabled(true)
    }
  }

  private inner class MyChrome : WebChromeClient() {
    private var mCustomView: View? = null
    private var mCustomViewCallback: CustomViewCallback? = null
    private var mOriginalOrientation = 0
    private var mOriginalSystemUiVisibility = 0

    override fun getDefaultVideoPoster(): Bitmap? {
      return if (mCustomView == null) {
        null
      } else {
        BitmapFactory.decodeResource(applicationContext.resources, 2130837573)
      }
    }

    override fun onHideCustomView() {
      (window.decorView as FrameLayout).removeView(mCustomView)
      mCustomView = null
      window.decorView.systemUiVisibility = mOriginalSystemUiVisibility
      requestedOrientation = mOriginalOrientation
      mCustomViewCallback?.onCustomViewHidden()
      mCustomViewCallback = null
    }

    override fun onShowCustomView(paramView: View?, paramCustomViewCallback: CustomViewCallback?) {
      if (mCustomView != null) {
        onHideCustomView()
        return
      }

      mCustomView = paramView
      mOriginalSystemUiVisibility = window.decorView.systemUiVisibility
      mOriginalOrientation = requestedOrientation
      mCustomViewCallback = paramCustomViewCallback
      (window.decorView as FrameLayout).addView(mCustomView, FrameLayout.LayoutParams(-1, -1))
      window.decorView.systemUiVisibility = 3846 or View.SYSTEM_UI_FLAG_LAYOUT_STABLE
    }
  }

  private companion object {
    private const val PORT = 3000
  }
}