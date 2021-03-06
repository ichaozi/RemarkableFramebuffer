[![RustPoC](https://i.imgur.com/c9YCAsy.jpg)](https://i.imgur.com/c9YCAsy.jpg)

[![PoC](https://thumbs.gfycat.com/WelltodoImprobableAlligator-size_restricted.gif)](https://gfycat.com/gifs/detail/WelltodoImprobableAlligator)

Documentation work is now complete. Everything from low latency partial updates to the eInk display to at least minimal multitouch, physical button and Wacom Digitizer input is now understood and can be found one way or another in this repository, either as code or as documentation.

The focus of this repository is now going to be the Rust library for providing these features. Potentially a `piston` backend might be created for `Remarkable`, allowing the use of `conrod` to simplify UI creation.

In cases where Rust implementation seems to contradict with the C implementation, the former can be taken as the source of truth as the `libremarkable` C-implementation was the first-pass that came to being during the exploration stage.

## Documenting the Undocumented Remarkable Low Latency I/O

This repository contains a collection of scripts, code and general information on what makes Remarkable Paper Tablet tick, focusing on gaining access to the low latency refresh capabilities of the device which are normally not exposed.

[![PoC](https://thumbs.gfycat.com/KindFearlessBigmouthbass-size_restricted.gif)](https://gfycat.com/gifs/detail/KindFearlessBigmouthbass)

[![PoC](https://thumbs.gfycat.com/GlitteringShortIchneumonfly-size_restricted.gif)](https://gfycat.com/gifs/detail/GlitteringShortIchneumonfly)

(GIF Preview has limited FPS -- click to watch at full framerate)

### Build Instructions
First, initialize the libfreetype submodules with `git submodule init && git submodule update`.

Then, run `make freetype` to generate the `libfreetype` static build with the expected flags.

Execute `make all` to generate the `poc` executable along with `spy.so`, `libremarkable.so`, `libremarkable.a` and `evtest`.

The makefiles assume the following are available in your `$PATH`, you may need to override or change them if they are installed elsewhere on your system:
```
CC = arm-linux-gnueabihf-gcc
CXX = arm-linux-gnueabihf-g++
AR = arm-linux-gnueabihf-ar
```
The toolchain that would be acquired from either of these sources would be able to cross-compile for the Remarkable Tablet:
```
AUR:         https://aur.archlinux.org/packages/arm-linux-gnueabihf-gcc/
Remarkable:  https://remarkable.engineering/deploy/sdk/poky-glibc-x86_64-meta-toolchain-qt5-cortexa9hf-neon-toolchain-2.1.3.sh
```

### Partial Redraw Proof of Concept (poc)
Contains the proof of concept for directly interacting with the eInk display driver to perform partial updates.

The key finding here is the magic values and their usage in conjunction with the dumped `mxcfb_*` data structures. Simply update the framebuffer and then call `ioctl` on the `/dev/fb0` FD with `REMARKABLE_PREFIX | MXCFB_SEND_UPDATE` in order to quickly the redraw region defined by `data.update_region` and that region only.

```c
#define REMARKABLE_PREFIX                       0x40484600
#define MXCFB_SEND_UPDATE                       0x0000002e
#define MXCFB_WAIT_FOR_VSYNC                    0x00000020
#define MXCFB_SET_GBL_ALPHA                     0x00000021
#define MXCFB_SET_CLR_KEY                       0x00000022
#define MXCFB_SET_OVERLAY_POS                   0x00000024
#define MXCFB_GET_FB_IPU_CHAN                   0x00000025
#define MXCFB_SET_LOC_ALPHA                     0x00000026
#define MXCFB_SET_LOC_ALP_BUF                   0x00000027
#define MXCFB_SET_GAMMA                         0x00000028
#define MXCFB_GET_FB_IPU_DI                     0x00000029
#define MXCFB_GET_DIFMT                         0x0000002a
#define MXCFB_GET_FB_BLANK                      0x0000002b
#define MXCFB_SET_WAVEFORM_MODES                0x0000002b
#define MXCFB_SET_DIFMT                         0x0000002c
#define MXCFB_SET_TEMPERATURE                   0x0000002c
#define MXCFB_SET_AUTO_UPDATE_MODE              0x0000002d
#define MXCFB_WAIT_FOR_UPDATE_COMPLETE	        0x0000002f
#define MXCFB_SET_PWRDOWN_DELAY	                0x00000030
#define MXCFB_GET_PWRDOWN_DELAY	                0x00000031
#define MXCFB_SET_UPDATE_SCHEME                 0x00000032
#define MXCFB_SET_MERGE_ON_WAVEFORM_MISMATCH    0x00000037

mxcfb_update_data data;
data.update_region.top = 0;
data.update_region.left = 0;
data.update_region.width = vinfo.xres;
data.update_region.height = vinfo.yres;
data.waveform_mode = 0x0002;
data.temp = 0x1001;
data.update_mode = 0x0000;
data.update_marker = 0x002a;
data.flags = 0;
data.alt_buffer_data = NULL;
ioctl(fb, REMARKABLE_PREFIX | MXCFB_SEND_UPDATE, &data);
```

### Framebuffer Overview
Current framebuffer can be dumped with:
```bash
ssh root@10.11.99.1 "cat /dev/fb0" | convert -depth 16 -size 1408x1872+0 gray:- png:/tmp/frame;
```

Remarkable Paper Tablet has an undocumented API for partial refreshes on its eInk display, which is what's behind its magic that disappears when custom Qt applications are used to draw on the screen, even using the toolchain provided by Remarkable.

The `xochitl` program opens `/dev/fb0`, which always ends up being the `FD=3`. It then writes to this FD when it wants to update the screen and uses primarily the following `ioctl` call in order to perform its partial updates when the user draws on the device (`0x4048462e` is the `PARTIAL_UPDATE_MAGIC`, and the next argument is a pointer to `mxcfb_update_data`):

What is particularly interesting here is that `0x4048462e` also happens to be Kindle's `MXCFB_SEND_UPDATE` magic. Something to keep in mind.

```c
typedef struct {
  uint32_t top;    // 0x0000
  uint32_t left;   // 0x0000 
  uint32_t width;  // 0x0258 = 600
  uint32_t height; // 0x0320 = 800
} mxcfb_rect;

typedef struct {
  mxcfb_rect update_region;

  uint32_t waveform_mode;  // 0x0002 = WAVEFORM_MODE_GC16 
  uint32_t update_mode;    // 0x0000 = UPDATE_MODE_PARTIAL 
  uint32_t update_marker;  // 0x002a 
  
  int temp;   // 0x1001 = TEMP_USE_PAPYRUS
  uint flags; // 0x0000 
  
  void* alt_buffer_data; // must not used when flags is 0
  ...
} mxcfb_update_data;

ioctl(3, 0x4048462e, 0x7ea2d290{
   updateRegion: x: 0
                 y: 0
                 width: 1404
                 height: 1872
   waveformMode: 3,
   updateMode:   0
   updateMarker: 46
   temp: 4096
   flags: 0000
   alt_buffer_data: 0x300f30
   ...
}) == 0
```

The `xochitl` program is statically linked with the `QsgEpaperPlugin` which can be found in this repository with the filename `libqsgepaper.a`. These implementations contained withing that library, however, are not used in the PoC as they are not yet fully explored and entirely undocumented. What is used instead is skipping what `libqsgepaper` can achieve with its undocumented portions listed at `xochitl-reference/libqsgepaper.md` and instead gaining lower level access to the hardware.

However, looking at the function signatures and the analysis so far, it looks like the PoC actually has gotten them right (`EPFrameBuffer::WaveformMode, EPFrameBuffer::UpdateMode` in `EPFramebuffer::sendUpdate`, returning a `uint32_t refresh_marker` that is referred to as an `updateCounter` in `epframebuffer.o`). The list of prototypes can be found at the end of this page.

Additionally take a look at the `/usr/bin/remarkable-test` program. It definitely offers interesting insights into the device.

### FrameBuffer Spy
A shared library that intercepts and displays undocumented framebuffer refresh `ioctl` calls for the Remarkable Paper Tablet. Usage:
```bash
$ systemctl stop xochitl
$ LD_PRELOAD=./spy.so xochitl
...
12:06.842 DebugHelperClass    	 void DocumentWorker::loadCachedPage(int) 191 ms (~DebugHelperClass() ../git/src/debug.h:16)
ioctl(3, 0x4048462e, 0x7ea2d290{
   updateRegion: x: 0
                 y: 0
                 width: 1404
                 height: 1872
   waveformMode: 3,
   updateMode:   0
   updateMarker: 45
   temp: 4096
   flags: 0000
   alt_buffer_data: 0x300f30
   ...
}) == 0
12:07.207 DebugHelperClass    	 void DocumentWorker::loadCachedPage(int) 364 ms (~DebugHelperClass() ../git/src/debug.h:16)
12:07.384 DebugHelperClass    	 void DocumentWorker::loadCachedPage(int) 175 ms (~DebugHelperClass() ../git/src/debug.h:16)
12:07.548 DebugHelperClass    	 void DocumentWorker::loadCachedPage(int) 162 ms (~DebugHelperClass() ../git/src/debug.h:16)
12:07.705 DebugHelperClass    	 void DocumentWorker::loadCachedPage(int) 155 ms (~DebugHelperClass() ../git/src/debug.h:16)
ioctl(3, 0x4048462e, 0x7ea2d290{
   updateRegion: x: 0
                 y: 0
                 width: 1404
                 height: 1872
   waveformMode: 3,
   updateMode:   0
   updateMarker: 46
   temp: 4096
   flags: 0000
   alt_buffer_data: 0x300f30
   ...
}) == 0
```

### Reading from Wacom Digitizer, Touch Screen and the physical buttons
The device features an ARM SoC from the i.MX6 family by Freescale (--> NXP --> Qualcomm) alongside a Parade TrueTouch Gen5 multitouch screen. The EPaper Display seems to be referred to as `es103cs1`. The firmware for this display can be found at `kernel-mxc-epdc-fb-reference/epdc_ES103CS1.fw`.
```
      epdc@020f4000 {
        compatible = "fsl,imx6sl-epdc", "fsl,imx6dl-epdc";
        reg = <0x20f4000 0x4000>;
        interrupts = <0x0 0x61 0x4>;
        clocks = <0x2 0x70 0x2 0x73>;
        clock-names = "epdc_axi", "epdc_pix";
        pinctrl-names = "default";
        pinctrl-0 = <0x10>;
        VCOM-supply = <0x11>;
        DISPLAY-supply = <0x12>;
        TMST-supply = <0x13>;
        status = "okay";
      };
```
```bash
remarkable: ~/ cat /proc/device-tree/model
reMarkable Prototype 1

remarkable: ~/ cat /proc/device-tree/compatible 
remarkable,zero-gravitasfsl,imx6sl

remarkable: ~/ cat /proc/bus/input/devices 
I: Bus=0018 Vendor=056a Product=0000 Version=0036
N: Name="Wacom I2C Digitizer"
P: Phys=
S: Sysfs=/devices/soc0/soc/2100000.aips-bus/21a4000.i2c/i2c-1/1-0009/input/input0
U: Uniq=
H: Handlers=mouse0 event0 
B: PROP=0
B: EV=b
B: KEY=1c03 0 0 0 0 0 0 0 0 0 0
B: ABS=f000003

I: Bus=0000 Vendor=0000 Product=0000 Version=0000
N: Name="cyttsp5_mt"
P: Phys=2-0024/input0
S: Sysfs=/devices/soc0/soc/2100000.aips-bus/21a8000.i2c/i2c-2/2-0024/input/input1
U: Uniq=
H: Handlers=event1 
B: PROP=2
B: EV=f
B: KEY=0
B: REL=0
B: ABS=6f38000 2000000

I: Bus=0019 Vendor=0001 Product=0001 Version=0100
N: Name="gpio-keys"
P: Phys=gpio-keys/input0
S: Sysfs=/devices/soc0/gpio-keys/input/input2
U: Uniq=
H: Handlers=kbd event2 
B: PROP=0
B: EV=3
B: KEY=8000 100640 0 0 0

remarkable: ~/ ls -latr /dev/input/
lrwxrwxrwx    1 root     root             6 Feb 23 05:52 touchscreen0 -> event0
crw-rw----    1 root     input      13,  32 Feb 23 05:52 mouse0
crw-rw----    1 root     input      13,  66 Feb 23 05:52 event2
crw-rw----    1 root     input      13,  65 Feb 23 05:52 event1
crw-rw----    1 root     input      13,  64 Feb 23 05:52 event0
drwxr-xr-x    2 root     root           120 Feb 23 05:52 by-path
drwxr-xr-x    3 root     root           180 Feb 23 05:52 .
crw-rw----    1 root     input      13,  63 Feb 23 05:52 mice
drwxr-xr-x    8 root     root          3460 Feb 23 09:30 ..
```
The firmware blob is available for this device however it isn't included in this repo since it is rather standard. Events from the digitizer can be seen by reading from these devices. Using the `evtest` like shown below:

#### /dev/input/event0 (Wacom I2C Digitizer)
- Only for input via the pen
- With and without contact
- Pressure sensitive, tilt-capable
```
      i2c@021a4000 {
        #address-cells = <0x1>;
        #size-cells = <0x0>;
        compatible = "fsl,imx6sl-i2c", "fsl,imx21-i2c";
        reg = <0x21a4000 0x4000>;
        interrupts = <0x0 0x25 0x4>;
        clocks = <0x2 0x6b>;
        status = "okay";
        clock-frequency = <0x186a0>;
        pinctrl-names = "default";
        pinctrl-0 = <0x28>;

        wacom-i2c@09 {
          compatible = "wacom,wacom-i2c";
          reg = <0x9>;
          interrupt-parent = <0x27>;
          interrupts = <0xa 0x2>;
          resets = <0x29>;
        };
      };
```
```bash
remarkable: ~/ ./evtest /dev/input/event0
Input driver version is 1.0.1
Input device ID: bus 0x18 vendor 0x56a product 0x0 version 0x36
Input device name: "Wacom I2C Digitizer"
Supported events:
  Event type 0 (Sync)
  Event type 1 (Key)
    Event code 320 (ToolPen)
    Event code 321 (ToolRubber)
    Event code 330 (Touch)
    Event code 331 (Stylus)
    Event code 332 (Stylus2)
  Event type 3 (Absolute)
    Event code 0 (X)
      Value   7509
      Min        0
      Max    20967
    Event code 1 (Y)
      Value  11277
      Min        0
      Max    15725
    Event code 24 (Pressure)
      Value      0
      Min        0
      Max     4095
    Event code 25 (Distance)
      Value     62
      Min        0
      Max      255
    Event code 26 (XTilt)
      Value      0
      Min    -9000
      Max     9000
    Event code 27 (YTilt)
      Value      0
      Min    -9000
      Max     9000
Testing ... (interrupt to exit)
Event: time 1519455612.131963, -------------- Report Sync ------------
Event: time 1519455612.138317, type 3 (Absolute), code 0 (X), value 6536
Event: time 1519455612.138317, type 3 (Absolute), code 25 (Distance), value 49
Event: time 1519455612.138317, -------------- Report Sync ------------
Event: time 1519455612.141944, type 3 (Absolute), code 0 (X), value 6562
Event: time 1519455612.141944, type 3 (Absolute), code 1 (Y), value 9457
Event: time 1519455612.141944, type 3 (Absolute), code 25 (Distance), value 54
Event: time 1519455612.141944, -------------- Report Sync ------------
Event: time 1519455612.148315, type 3 (Absolute), code 0 (X), value 6590
Event: time 1519455612.148315, type 3 (Absolute), code 1 (Y), value 9456
Event: time 1519455612.148315, type 3 (Absolute), code 25 (Distance), value 60
Event: time 1519455612.148315, -------------- Report Sync ------------
Event: time 1519455612.151963, type 3 (Absolute), code 0 (X), value 6617
Event: time 1519455612.151963, type 3 (Absolute), code 1 (Y), value 9454
Event: time 1519455612.151963, type 3 (Absolute), code 25 (Distance), value 67
Event: time 1519455612.151963, -------------- Report Sync ------------
Event: time 1519455612.158320, type 3 (Absolute), code 0 (X), value 6645
Event: time 1519455612.158320, type 3 (Absolute), code 1 (Y), value 9452
Event: time 1519455612.158320, type 3 (Absolute), code 25 (Distance), value 74
Event: time 1519455612.158320, -------------- Report Sync ------------
Event: time 1519455612.161156, type 3 (Absolute), code 0 (X), value 6669
Event: time 1519455612.161156, type 3 (Absolute), code 1 (Y), value 9449
Event: time 1519455612.161156, -------------- Report Sync ------------
Event: time 1519455612.167466, type 3 (Absolute), code 0 (X), value 6688
Event: time 1519455612.167466, type 3 (Absolute), code 1 (Y), value 9446
Event: time 1519455612.167466, -------------- Report Sync ------------
Event: time 1519455612.181232, type 1 (Key), code 320 (ToolPen), value 0
Event: time 1519455612.181232, -------------- Report Sync ------------
```

#### /dev/input/event1 (cyttsp5_mt 'multitouch')
The device features a Parade TrueTouch Gen5 multitouch screen. The firmware for the MT controller can be found at `kernel-mxc-epdc-fb-reference/cyttsp5_fw_pid00.bin`.
```
cy,mt {
       cy,name = "cyttsp5_mt";

       cy,inp_dev_name = "cyttsp5_mt";
       cy,flags = <0>;
       cy,abs =
         /* ABS_MT_POSITION_X, CY_ABS_MIN_X, CY_ABS_MAX_X, 0, 0 */
         <0x35 0 880 0 0
         /* ABS_MT_POSITION_Y, CY_ABS_MIN_Y, CY_ABS_MAX_Y, 0, 0 */
         0x36 0 1280 0 0
         /* ABS_MT_PRESSURE, CY_ABS_MIN_P, CY_ABS_MAX_P, 0, 0 */
         0x3a 0 255 0 0
         /* CY_IGNORE_VALUE, CY_ABS_MIN_W, CY_ABS_MAX_W, 0, 0 */
         0xffff 0 255 0 0
         /* ABS_MT_TRACKING_ID, CY_ABS_MIN_T, CY_ABS_MAX_T, 0, 0 */
         0x39 0 15 0 0
         /* ABS_MT_TOUCH_MAJOR, 0, 255, 0, 0 */
         0x30 0 255 0 0
         /* ABS_MT_TOUCH_MINOR, 0, 255, 0, 0 */
         0x31 0 255 0 0
         /* ABS_MT_ORIENTATION, -127, 127, 0, 0 */
         0x34 0xffffff81 127 0 0
         /* ABS_MT_TOOL_TYPE, 0, MT_TOOL_MAX, 0, 0 */
         0x37 0 1 0 0
         /* ABS_DISTANCE, 0, 255, 0, 0 */
         0x19 0 255 0 0>;

       cy,vkeys_x = <720>;
       cy,vkeys_y = <1280>;

       cy,virtual_keys = /* KeyCode CenterX CenterY Width Height */
         /* KEY_BACK */
         <158 1360 90 160 180
         /* KEY_MENU */
         139 1360 270 160 180
         /* KEY_HOMEPAGE */
         172 1360 450 160 180
         /* KEY SEARCH */
         217 1360 630 160 180>;
     };
```

```bash
remarkable: ~/ ./evtest /dev/input/event1
Input driver version is 1.0.1
Input device ID: bus 0x0 vendor 0x0 product 0x0 version 0x0
Input device name: "cyttsp5_mt"
Supported events:
  Event type 0 (Sync)
  Event type 1 (Key)
  Event type 2 (Relative)
  Event type 3 (Absolute)
    Event code 25 (Distance)
      Value      0
      Min        0
      Max      255
    Event code 47 (?)
      Value      0
      Min        0
      Max       31
    Event code 48 (?)
      Value      0
      Min        0
      Max      255
    Event code 49 (?)
      Value      0
      Min        0
      Max      255
    Event code 52 (?)
      Value      0
      Min     -127
      Max      127
    Event code 53 (?)
      Value      0
      Min        0
      Max      767
    Event code 54 (?)
      Value      0
      Min        0
      Max     1023
    Event code 55 (?)
      Value      0
      Min        0
      Max        1
    Event code 57 (?)
      Value      0
      Min        0
      Max    65535
    Event code 58 (?)
      Value      0
      Min        0
      Max      255
Testing ... (interrupt to exit)
Event: time 1519456007.622705, type 3 (Absolute), code 57 (?), value 139
Event: time 1519456007.622705, type 3 (Absolute), code 53 (?), value 186
Event: time 1519456007.622705, type 3 (Absolute), code 54 (?), value 323
Event: time 1519456007.622705, type 3 (Absolute), code 58 (?), value 117
Event: time 1519456007.622705, type 3 (Absolute), code 48 (?), value 9
Event: time 1519456007.622705, type 3 (Absolute), code 52 (?), value 3
Event: time 1519456007.622705, -------------- Report Sync ------------
Event: time 1519456007.667954, type 3 (Absolute), code 57 (?), value -1
Event: time 1519456007.667954, -------------- Report Sync ------------
Event: time 1519456008.162604, type 3 (Absolute), code 57 (?), value 140
Event: time 1519456008.162604, type 3 (Absolute), code 53 (?), value 222
Event: time 1519456008.162604, type 3 (Absolute), code 54 (?), value 509
Event: time 1519456008.162604, type 3 (Absolute), code 58 (?), value 75
Event: time 1519456008.162604, type 3 (Absolute), code 48 (?), value 4
Event: time 1519456008.162604, type 3 (Absolute), code 52 (?), value 2
Event: time 1519456008.162604, -------------- Report Sync ------------
Event: time 1519456008.245695, type 3 (Absolute), code 57 (?), value -1
Event: time 1519456008.245695, -------------- Report Sync ------------
```

#### /dev/input/event2 (Reading Physical Buttons)
```bash
remarkable: ~/ ./evtest /dev/input/event2
Input driver version is 1.0.1
Input device ID: bus 0x19 vendor 0x1 product 0x1 version 0x100
Input device name: "gpio-keys"
Supported events:
  Event type 0 (Sync)
  Event type 1 (Key)
    Event code 102 (Home)
    Event code 105 (Left)
    Event code 106 (Right)
    Event code 116 (Power)
    Event code 143 (WakeUp)
Testing ... (interrupt to exit)
Event: time 1519456176.426978, type 1 (Key), code 106 (Right), value 1
Event: time 1519456176.426978, -------------- Report Sync ------------
Event: time 1519456176.706831, type 1 (Key), code 106 (Right), value 0
Event: time 1519456176.706831, -------------- Report Sync ------------
```
