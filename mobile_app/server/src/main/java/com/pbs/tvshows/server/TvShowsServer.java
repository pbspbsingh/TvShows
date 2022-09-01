package com.pbs.tvshows.server;

import java.util.concurrent.atomic.AtomicBoolean;

public class TvShowsServer {

  static {
    System.loadLibrary("server");
  }

  private TvShowsServer() {}

  private static final AtomicBoolean hasStarted = new AtomicBoolean(false);

  private static native String startServer(
      String cacheFolder,
      int asyncThread,
      int blockingThread,
      int port
  );

  public static synchronized void startServerInBackground(String cacheFolder, int port) {
    if (hasStarted.compareAndSet(false, true)) {
      new Thread(() -> {
        String message = startServer(cacheFolder, 2, 1, port);
        System.out.println("Server: " + message);
      }, "TvShowsServer").start();
    } else {
      System.out.println("Server is already running.");
    }
  }
}
