# wgserve
Android application to route incoming connections from Wireguard tunnels to device's network without root access.
Based on Boringtun Wireguard implementation and smoltcp user-space TCP/IP stack.

Basically it is [wgslirpy](https://github.com/vi/wgslirpy) tool, but as an Android app.

# Features

(see https://github.com/vi/wgslirpy#features)

# Limitations

* Inconvenient, config-file based UI.
* No status of what is happening inside (besides logcat)

# Usage steps

1. Build the app from source code or download it from releases
2. Launch
3. Press "Sample config". Obviously, do not use that private key for real.
4. Adjust IP addresses and keys. Remove unnesesary settings. When in doubt, experiment with `wgslirpy` CLI tool first to familiarize yourself with the options.
5. Copy and paste the config somewhere.
6. Transform it (manually) to wg-quick config, apply it somewhere, or turn into Qr code to use on another Android device.
7. Press "start". Watch logcat.
8. Ignore "connection expired" or "no current session" errors, it keeps trying connecting.
9. Check other peer for Wireguard tunnel status. There is currently no log message meaning "yes, now I'm connected" (unless `debug = true`).
10. Check if `pinger` is working (if configured)
11. Check if usual connectivity is working.

# Examples of reasons why it can fail to work

* Android device is in another network
* Firewall on Android device blocking traffic of the application
* You confused private and public key in configs. Or confused local and peer keys.
* Allowed ips in counterpart's configuration does not allow traffic
* No keepalive interval on both sides of the connection
* No IP address added to counterpart's Wireguard interface
* DNS not configured
* No default route added
* Android device decided to go to sleep (and you have not configured exception for power optimisations for WgServer).
