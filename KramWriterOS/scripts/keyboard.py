import RPi.GPIO as GPIO
import time
import uinput

# 1. PHYSICAL CONFIGURATION
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. KEYMAPS (Using direct dictionaries to avoid IndexError)
_BASE = {
    (0,0): uinput.KEY_TAB,  (0,1): uinput.KEY_Q, (0,2): uinput.KEY_W, (0,3): uinput.KEY_E, (0,4): uinput.KEY_R, (0,5): uinput.KEY_T, (0,6): uinput.KEY_Y, (0,7): uinput.KEY_U, (0,8): uinput.KEY_I, (0,9): uinput.KEY_O, (0,10): uinput.KEY_P, (0,11): uinput.KEY_BACKSPACE,
    (1,0): uinput.KEY_ESC,  (1,1): uinput.KEY_A, (1,2): uinput.KEY_S, (1,3): uinput.KEY_D, (1,4): uinput.KEY_F, (1,5): uinput.KEY_G, (1,6): uinput.KEY_H, (1,7): uinput.KEY_J, (1,8): uinput.KEY_K, (1,9): uinput.KEY_L, (1,10): uinput.KEY_SEMICOLON, (1,11): uinput.KEY_ENTER,
    (2,0): uinput.KEY_LEFTSHIFT, (2,1): uinput.KEY_Z, (2,2): uinput.KEY_X, (2,3): uinput.KEY_C, (2,4): uinput.KEY_V, (2,5): uinput.KEY_B, (2,6): uinput.KEY_N, (2,7): uinput.KEY_M, (2,8): uinput.KEY_COMMA, (2,9): uinput.KEY_DOT, (2,10): uinput.KEY_UP, (2,11): uinput.KEY_SLASH,
    (3,0): uinput.KEY_LEFTCTRL, (3,1): uinput.KEY_LEFTMETA, (3,2): uinput.KEY_LEFTALT, (3,3): "NUM", (3,4): uinput.KEY_SPACE, (3,5): uinput.KEY_SPACE, (3,6): "SYM", (3,7): uinput.KEY_LEFT, (3,8): uinput.KEY_DOWN, (3,9): uinput.KEY_RIGHT
}

_NUM = {
    (0,1): uinput.KEY_1, (0,2): uinput.KEY_2, (0,3): uinput.KEY_3, (0,4): uinput.KEY_4, (0,5): uinput.KEY_5, (0,6): uinput.KEY_6, (0,7): uinput.KEY_7, (0,8): uinput.KEY_8, (0,9): uinput.KEY_9, (0,10): uinput.KEY_0,
    (1,6): uinput.KEY_LEFT, (1,7): uinput.KEY_DOWN, (1,8): uinput.KEY_UP, (1,9): uinput.KEY_RIGHT
}

_SYM = {
    (1,1): uinput.KEY_GRAVE, (1,3): uinput.KEY_LEFTBRACE, (1,4): uinput.KEY_RIGHTBRACE, (1,5): uinput.KEY_BACKSLASH, (1,6): uinput.KEY_MINUS, (1,7): uinput.KEY_EQUAL, (1,8): uinput.KEY_LEFTBRACE, (1,9): uinput.KEY_RIGHTBRACE, (1,10): uinput.KEY_BACKSLASH
}

# 3. INITIALIZE DEVICE
all_possible_keys = set()
for layer in [_BASE, _NUM, _SYM]:
    for key in layer.values():
        if isinstance(key, int): all_possible_keys.add(key)

device = uinput.Device(list(all_possible_keys))
time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

# 4. TRACKING
pressed_keys = {} # (r, c): uinput_key_sent
active_cols = set() # For Vertical Lockout (Ghosting fix)
current_layers = {"NUM": False, "SYM": False}

print("KramWriter: Manual Run Starting...")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            time.sleep(0.0002) # Settle time
            
            for c_idx, c_pin in enumerate(COLS):
                key_id = (r_idx, c_idx)
                is_down = (GPIO.input(c_pin) == GPIO.LOW)
                
                # Check for Layer Modifiers (NUM/SYM)
                base_val = _BASE.get(key_id)
                if base_val in ["NUM", "SYM"]:
                    current_layers[base_val] = is_down
                    continue

                # KEY DOWN
                if is_down and key_id not in pressed_keys:
                    # Ignore if column is already used (Vertical Lockout)
                    if c_idx not in active_cols:
                        # Determine key based on layer priority
                        target_key = None
                        if current_layers["NUM"]: target_key = _NUM.get(key_id)
                        elif current_layers["SYM"]: target_key = _SYM.get(key_id)
                        
                        # Fallback to Base Layer if no key found in layer
                        if target_key is None: target_key = _BASE.get(key_id)
                        
                        if isinstance(target_key, int):
                            device.emit(target_key, 1)
                            pressed_keys[key_id] = target_key
                            active_cols.add(c_idx)
                            print(f"Down: {key_id}")

                # KEY UP
                elif not is_down and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0)
                    active_cols.remove(c_idx)
                    print(f"Up: {key_id}")
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)

except KeyboardInterrupt:
    GPIO.cleanup()