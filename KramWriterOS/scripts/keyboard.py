import RPi.GPIO as GPIO
import time
import uinput

# 1. PINS
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. FLAT KEYMAP (Alpha only, mapped to your 12x4 grid)
# I've filled the first 26 slots with letters. 
# Any press on these specific grid spots should type that letter.
ALPHA_MAP = [
    [uinput.KEY_Q, uinput.KEY_W, uinput.KEY_E, uinput.KEY_R, uinput.KEY_T, uinput.KEY_Y, uinput.KEY_U, uinput.KEY_I, uinput.KEY_O, uinput.KEY_P, None, None],
    [uinput.KEY_A, uinput.KEY_S, uinput.KEY_D, uinput.KEY_F, uinput.KEY_G, uinput.KEY_H, uinput.KEY_J, uinput.KEY_K, uinput.KEY_L, None, None, None],
    [uinput.KEY_Z, uinput.KEY_X, uinput.KEY_C, uinput.KEY_V, uinput.KEY_B, uinput.KEY_N, uinput.KEY_M, None, None, None, None, None],
    [None, None, None, None, None, None, None, None, None, None, None, None]
]

# 3. INITIALIZE DEVICE
# We register all 26 letters
device = uinput.Device([
    uinput.KEY_A, uinput.KEY_B, uinput.KEY_C, uinput.KEY_D, uinput.KEY_E, 
    uinput.KEY_F, uinput.KEY_G, uinput.KEY_H, uinput.KEY_I, uinput.KEY_J, 
    uinput.KEY_K, uinput.KEY_L, uinput.KEY_M, uinput.KEY_N, uinput.KEY_O, 
    uinput.KEY_P, uinput.KEY_Q, uinput.KEY_R, uinput.KEY_S, uinput.KEY_T, 
    uinput.KEY_U, uinput.KEY_V, uinput.KEY_W, uinput.KEY_X, uinput.KEY_Y, 
    uinput.KEY_Z
])

time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

# 4. TRACKING
pressed_keys = {} # To prevent repeating letters

print("ALPHA TEST: Typing A-Z based on grid position...")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                
                is_down = (GPIO.input(c_pin) == GPIO.LOW)
                key_id = (r_idx, c_idx)

                if is_down and key_id not in pressed_keys:
                    # Get the key from our flat map
                    key = ALPHA_MAP[r_idx][c_idx]
                    
                    if key is not None:
                        print(f"Hit! Grid {r_idx},{c_idx} -> Typing Key")
                        device.emit(key, 1) # Press
                        pressed_keys[key_id] = key
                
                elif not is_down and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0) # Release
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)

except KeyboardInterrupt:
    GPIO.cleanup()