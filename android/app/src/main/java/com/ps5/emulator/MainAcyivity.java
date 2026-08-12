package com.ps5.emulator;

import android.app.Activity;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;

public class MainActivity extends Activity {
    
    static {
        System.loadLibrary("ps5_emulator");
    }
    
    private TextView statusText;
    private Button startButton;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        statusText = findViewById(R.id.statusText);
        startButton = findViewById(R.id.startButton);
        
        startButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                String result = startEmulator("/sdcard/games/");
                statusText.setText(result);
            }
        });
        
        String version = getVersion();
        statusText.setText("PS5 Emulator " + version);
    }
    
    // Native methods
    private native String startEmulator(String gamePath);
    private native void stopEmulator();
    private native String getVersion();
}
