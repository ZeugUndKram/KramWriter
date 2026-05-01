import RPi.GPIO as GPIO
import time
import uinput

# 1. Define the physical GPIO pins
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. Define the Virtual Keyboard and the keys it is allowed to "press"
# You can add more keys here (e.g., uinput.KEY_F1, etc.)
device = uinput.Device([
    uinput.KEY_ESC, uinput.KEY_1, uinput.KEY_2, uinput.KEY_3, uinput.KEY_4, 
    uinput.KEY_5, uinput.KEY_6, uinput.KEY_7, uinput.KEY_8, uinput.KEY_9, 
    uinput.KEY_0, uinput.KEY_BACKSPACE, uinput.KEY_TAB, uinput.KEY_Q, 
    uinput.KEY_W, uinput.KEY_E, uinput.KEY_R, uinput.KEY_T, uinput.KEY_Y, 
    uinput.KEY_U, uinput.KEY_I, uinput.KEY_O, uinput.KEY_P, uinput.KEY_ENTER,
    uinput.KEY_LEFTSHIFT, uinput.KEY_A, uinput.KEY_S, uinput.KEY_D, uinput.KEY_F, 
    uinput.KEY_G, uinput.KEY_H, uinput.KEY_J, uinput.KEY_K, uinput.KEY_L, 
    uinput.KEY_SEMICOLON, uinput.KEY_RIGHTSHIFT, uinput.KEY_LEFTCTRL, 
    uinput.KEY_LEFTALT, uinput.KEY_LEFTMETA, uinput.KEY_SPACE, uinput.KEY_LEFT, 
    uinput.KEY_DOWN, uinput.KEY_UP, uinput.KEY_RIGHT
])

# 3. Map your matrix to those uinput keys
KEY_MAP = [
    [uinput.KEY_ESC, uinput.KEY_1, uinput.KEY_2, uinput.KEY_3, uinput.KEY_4, uinput.KEY_5, uinput.KEY_6, uinput.KEY_7, uinput.KEY_8, uinput.KEY_9, uinput.KEY_0, uinput.KEY_BACKSPACE],
    [uinput.KEY_TAB, uinput.KEY_Q, uinput.KEY_W, uinput.KEY_E, uinput.KEY_R, uinput.KEY_T, uinput.KEY_Y, uinput.KEY_U, uinput.KEY_I, uinput.KEY_O, uinput.KEY_P, uinput.KEY_ENTER],
    [uinput.KEY_LEFTSHIFT, uinput.KEY_A, uinput.KEY_S, uinput.KEY_D, uinput.KEY_F, uinput.KEY_G, uinput.KEY_H, uinput.KEY_J, uinput.KEY_K, uinput.KEY_L, uinput.KEY_SEMICOLON, uinput.KEY_RIGHTSHIFT],
    [uinput.KEY_LEFTCTRL, uinput.KEY_LEFTALT, uinput.KEY_LEFTMETA, uinput.KEY_SPACE, None, uinput.KEY_SPACE, None, uinput.KEY_LEFT, uinput.KEY_DOWN, uinput.KEY_UP, uinput.KEY_RIGHT, None]
]

GPIO.setmode(GPIO.BCM)
for c in COLS:
    GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS:
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

print("Writerdeck keyboard active...")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                if GPIO.input(c_pin) == GPIO.LOW:
                    key = KEY_MAP[r_idx][c_idx]
                    if key:
                        device.emit_click(key) # This sends press + release
                        # Wait for release to avoid accidental "aaaaaa"
                        while GPIO.input(c_pin) == GPIO.LOW:
                            time.sleep(0.02)
            GPIO.output(r_pin, GPIO.HIGH)
            time.sleep(0.01)
except KeyboardInterrupt:
    GPIO.cleanup()