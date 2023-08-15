package org.vi_server.wgserver;


import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.os.Build;
import android.os.Bundle;
import android.widget.Button;
import android.widget.EditText;
import android.widget.TextView;

public class MainActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Context ctx = this;

        {
            Button b = findViewById(R.id.bStart);
            b.setOnClickListener(view -> {
                Intent intent = new Intent(ctx, Serv.class);

                EditText t = findViewById(R.id.tConfig);
                String config = t.getText().toString();

                long instance = Native.create();
                String ret = Native.setConfig(instance, config);

                TextView s = findViewById(R.id.tStatus);
                if (ret != null && !ret.isEmpty()) {
                    s.setText(ret);
                } else {
                    intent.putExtra("instance", instance);

                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                        ctx.startForegroundService(intent);
                    } else {
                        ctx.startService(intent);
                    }
                    s.setText("started");
                }

            });
        }
        {
            Button b = findViewById(R.id.bStop);
            b.setOnClickListener(view -> {
                TextView s = findViewById(R.id.tStatus);
                Intent intent = new Intent(ctx, Serv.class);
                ctx.stopService(intent);
                s.setText("stopped");
            });
        }

        {
            Button b = findViewById(R.id.bSampleConfig);
            b.setOnClickListener(view -> {
                EditText t = findViewById(R.id.tConfig);
                t.setText(Native.getSampleConfig());
            });
        }
    }
}
