import RPi.GPIO as GPIO
import time
import uinput

# 1. PINS
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. FULL GRID MAP (Directly matching your 40% Ortho Layout)
# We fill this with actual keys. Use None for empty spots.
GRID_MAP = {
    (0,0): uinput.KEY_TAB,  (0,1): uinput.KEY_Q, (0,2): uinput.KEY_W, (0,3): uinput.KEY_E, (0,4): uinput.KEY_R, (0,5): uinput.KEY_T, (0,6): uinput.KEY_Y, (0,7): uinput.KEY_U, (0,8): uinput.KEY_I, (0,9): uinput.KEY_O, (0,10): uinput.KEY_P, (0,11): uinput.KEY_BACKSPACE,
    (1,0): uinput.KEY_ESC,  (1,1): uinput.KEY_A, (1,2): uinput.KEY_S, (1,3): uinput.KEY_D, (1,4): uinput.KEY_F, (1,5): uinput.KEY_G, (1,6): uinput.KEY_H, (1,7): uinput.KEY_J, (1,8): uinput.KEY_K, (1,9): uinput.KEY_L, (1,10): uinput.KEY_SEMICOLON, (1,11): uinput.KEY_ENTER,
    (2,0): uinput.KEY_LEFTSHIFT, (2,1): uinput.KEY_Z, (2,2): uinput.KEY_X, (2,3): uinput.KEY_C, (2,4): uinput.KEY_V, (2,5): uinput.KEY_B, (2,6): uinput.KEY_N, (2,7): uinput.KEY_M, (2,8): uinput.KEY_COMMA, (2,9): uinput.KEY_DOT, (2,10): uinput.KEY_UP, (2,11): uinput.KEY_SLASH,
    (3,4): uinput.KEY_SPACE, (3,5): uinput.KEY_SPACE # The 2u Spacebars
}

# 3. INITIALIZE DEVICE
all_keys = [k for k in GRID_MAP.values() if k is not None]
device = uinput.Device(all_keys)
time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

# 4. TRACKING
pressed_keys = {} 

print("KRAMWRITER: Alpha + Function Keys Active.")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                
                is_down = (GPIO.input(c_pin) == GPIO.LOW)
                key_id = (r_idx, c_idx)

                # KEY PRESSED
                if is_down and key_id not in pressed_keys:
                    key = GRID_MAP.get(key_id)
                    if key:
                        device.emit(key, 1) # Press
                        pressed_keys[key_id] = key
                        # Minimal debounce sleep
                        time.sleep(0.005) 
                
                # KEY RELEASED
                elif not is_down and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0) # Release
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        # Scan frequency control
        time.sleep(0.01)

except KeyboardInterrupt:
    GPIO.cleanup()