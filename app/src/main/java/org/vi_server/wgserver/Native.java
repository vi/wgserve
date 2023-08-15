package org.vi_server.wgserver;

public class Native {
    static {
        System.loadLibrary("wgserv");
    }

    public static native long create();

    /// Empty or null string = no error
    public static native String setConfig(long instance, String toml_config);

    /// Block thread and start it for real
    ///
    /// Empty or null string means no error
    public static native String run(long instance);

    /// Stop and deallocate the instance
    /// May be called from other thread
    public static native void destroy(long instance);

    public static native String getSampleConfig();
}
