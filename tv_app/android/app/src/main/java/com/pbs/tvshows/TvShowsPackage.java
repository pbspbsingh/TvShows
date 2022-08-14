package com.pbs.tvshows;

import androidx.annotation.NonNull;
import com.facebook.react.ReactPackage;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.uimanager.ViewManager;
import com.pbs.tvshows.keyboard.KeyBoardModule;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public class TvShowsPackage implements ReactPackage {

  @NonNull
  @Override
  public List<ViewManager> createViewManagers(@NonNull ReactApplicationContext reactContext) {
    return Collections.emptyList();
  }

  @NonNull
  @Override
  public List<NativeModule> createNativeModules(@NonNull ReactApplicationContext reactContext) {
    final List<NativeModule> modules = new ArrayList<>();
    modules.add(new KeyBoardModule(reactContext));
    return modules;
  }
}
