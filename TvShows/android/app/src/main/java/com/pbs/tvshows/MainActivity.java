package com.pbs.tvshows;

import android.os.Bundle;
import android.util.Log;
import android.view.KeyEvent;
import com.facebook.react.ReactActivity;
import com.pbs.tvshows.keyboard.KeyBoardModule;

public class MainActivity extends ReactActivity {

  @Override
  protected String getMainComponentName() {
    return "TvShows";
  }

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(null);
  }

  @Override
  public boolean onKeyDown(int keyCode, KeyEvent event) {
    KeyBoardModule.onKeyEvent(keyCode, event);
    return super.onKeyDown(keyCode, event);
  }
}
