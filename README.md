# Daydream Mouse

When I found an old [Google Daydream](https://en.wikipedia.org/wiki/Google_Daydream) phone vr headset at my thrift store, I grabbed it just because I thought it looked neat. Looking at the controller, though, I realized it'd be a nice little handheld clicker/mouse to have to interact with a pc at a distance.

I've never worked with BLE, or even much Rust, so I expect this to be extremely janky - but, it does work!

I've only tested this on my Debian computer with X11, YMMV on other systems.

Huge shoutout to [Matteo P. and this article](https://medium.com/hackernoon/how-i-hacked-google-daydream-controller-c4619ef318e4) about hacking the Daydream controller, without his findings this probably wouldn't have been possible for me.

## Dependencies
- [enigo](https://github.com/enigo-rs/enigo/tree/main) for input simulation
- [btleplug](https://github.com/deviceplug/btleplug/tree/master) to connect to the controller.

## Default Bindings
- Touchpad - Move mouse
- Touchpad Click - Left Click
- App Button - Right Click
- Home Button - Toggle Scroll Mode
    - While in scroll mode, iPod style circular scrolling on the touchpad scrolls the mouse.
    - Click home or the touchpad again to exit.
- Volume Up - Escape
- Volume Down - Meta/Command/Windows/Super Key

The IMU is currently ignored because I didn't want to have to figure out how to filter the motion data and get it working.