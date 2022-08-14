package com.pbs.tvshows.keyboard;

import android.util.Log;
import android.view.KeyEvent;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import java.lang.ref.WeakReference;

public class KeyBoardModule extends ReactContextBaseJavaModule {

  private static final String TAG = "KeyBoardModule";
  private static WeakReference<KeyBoardModule> weakRef = new WeakReference<>(null);

  public KeyBoardModule(@Nullable ReactApplicationContext reactContext) {
    super(reactContext);
    weakRef = new WeakReference<>(this);
  }

  @NonNull
  @Override
  public String getName() {
    return TAG;
  }

  public static void onKeyEvent(int keyCode, KeyEvent event) {
    Log.d(TAG, String.format("Keycode: %s, event: %s", keyCode, event));
    if (weakRef != null) {
      KeyBoardModule module = weakRef.get();
      if (module != null) {
        final WritableMap params = Arguments.createMap();
        params.putInt("keyCode", keyCode);
        params.putString("eventTime", String.valueOf(event.getEventTime()));
        module.getReactApplicationContext()
            .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
            .emit("keyEvent", params);
      }
    }
  }
}
