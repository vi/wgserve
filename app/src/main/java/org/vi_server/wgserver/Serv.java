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

public class Serv extends Service {
    private static final String CHANNEL_DEFAULT_IMPORTANCE = "default";
    private static final int ONGOING_NOTIFICATION_ID = 1;
    private static Notification.Builder nb;
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
        String config = intent.getStringExtra("config");

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

        boolean failed = false;

        // start here

        nb = new Notification.Builder(this)
                .setContentTitle(getText(R.string.app_name))
                .setContentText(notiftext)
                .setSmallIcon(R.drawable.wgserver)
                .setContentIntent(pendingIntent);

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            nb.setChannelId(CHANNEL_DEFAULT_IMPORTANCE);
        }
        if (failed) {
            nb.setContentText("Failed to start service");
        }
        Notification notification = nb.build();

        startForeground(ONGOING_NOTIFICATION_ID, notification);

        if (failed) {
            this.stopForeground(true);
            this.stopSelf();
            return super.onStartCommand(intent, flags, startId);
        }

        PowerManager pm = (PowerManager)this.getSystemService(
                Context.POWER_SERVICE);
        wl = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "wgserver:wl");
        wl.acquire();

        return super.onStartCommand(intent, flags, startId);
    }

    @Override
    public void onDestroy() {

        if (wl != null) {
            wl.release();
            wl = null;
        }
        super.onDestroy();
    }
}
