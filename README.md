# About the project
This is the code for a 6-axis/GPS based lap timer running on Linux.

The initial prototype is being developed using Rust on a Raspberry Pi 4, running Ubuntu-MATE desktop.
Then it will be ported to a compact linux SBC running a custom Linux build.

The 6-axis sensor used is an MPU-6050 and the GPS sensor a "blox NEO-6M-0-001" installed on a GPS6MV2 PCB

## Development Log
I currently have the 240x240 SPI LCD running on the Ubuntu (64-bit) Raspberry Pi 4 for development (so that I can use vscode remote-ssh which is not possible on the Pi Zero)
A minimal summary of the GPS data is displayed on the LCD, which has highlighted an issue with the current display driver: I'm using a non buffered, embedded display driver which means that text flickers as it's updated (first cleared and then rewritten with the new test). As I'm basing this on a linux SBC, I can afford to buffer and even use hardware-accelarated graphics. The main issue right now is the lack of buffering, but I also need to consider CPU usage (i.e. ensure I use DMA for SPI transfers).
I did a quick test to see how efficient the underlying driver is, and although I could blit the screen at a round 15fps (visual estimate) it was saturating the CPU.
It seems the way forward is to use a proper display driver (i.e. fbtft) that provides a frame buffer I can draw into and should be power-efficient.
