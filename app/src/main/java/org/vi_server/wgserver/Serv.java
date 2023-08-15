package org.vi_server.wgserver;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Context;
import android.content.Intent;
import android.os.Build;
import android.os.IBinder;
import android.os.PowerManager;
import android.util.Log;

public class Serv extends Service {
    private static final String CHANNEL_DEFAULT_IMPORTANCE = "default";
    private static final int ONGOING_NOTIFICATION_ID = 1;
    private static Notification.Builder nb;
    private static long instance = 0;
    private static PowerManager.WakeLock wl;

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    @Override
    public void onCreate() {
        super.onCreate();
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        if (instance != 0) {
            Native.destroy(instance);
        }
        instance = intent.getLongExtra("instance", 0);
        if (instance == 0) {
            this.stopForeground(true);
            this.stopSelf();
            return super.onStartCommand(intent, flags, startId);
        }

        Intent notificationIntent = new Intent(this, MainActivity.class);
        PendingIntent pendingIntent =
                PendingIntent.getActivity(this, 0, notificationIntent,
                        PendingIntent.FLAG_IMMUTABLE);


        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            int importance = NotificationManager.IMPORTANCE_LOW;
            NotificationChannel channel = new NotificationChannel(CHANNEL_DEFAULT_IMPORTANCE, "WgServer", importance);
            channel.setDescription("WgServer running");
            NotificationManager notificationManager = getSystemService(NotificationManager.class);
            notificationManager.createNotificationChannel(channel);
        }

        CharSequence notiftext = getText(R.string.service_desc);


        nb = new Notification.Builder(this)
                .setContentTitle(getText(R.string.app_name))
                .setContentText(notiftext)
                .setSmallIcon(R.drawable.wgserver)
                .setContentIntent(pendingIntent);

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            nb.setChannelId(CHANNEL_DEFAULT_IMPORTANCE);
        }

        Notification notification = nb.build();

        startForeground(ONGOING_NOTIFICATION_ID, notification);

        PowerManager pm = (PowerManager)this.getSystemService(
                Context.POWER_SERVICE);
        wl = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "wgserver:wl");
        wl.acquire();

        new Thread(() -> {
            String ret2 = Native.run(instance);
            if (ret2 == null || ret2.isEmpty()) {
                Log.i("WgServer","Background thread exited without signaling failure");
            } else {
                Log.w("WgServer", "Background thread existed with error: " + ret2);
            }
            this.stopSelf();
        }).start();

        return super.onStartCommand(intent, flags, startId);
    }

    @Override
    public void onDestroy() {

        if (wl != null) {
            wl.release();
            wl = null;
        }
        if (instance != 0) {
            Native.destroy(instance);
            instance=0;
        }
        super.onDestroy();
    }
}
