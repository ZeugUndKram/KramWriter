import RPi.GPIO as GPIO
import time
import uinput

# 1. PINS
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. KEYMAPS
# _BASE Layer
_BASE = {
    (0,0): uinput.KEY_TAB,  (0,1): uinput.KEY_Q, (0,2): uinput.KEY_W, (0,3): uinput.KEY_E, (0,4): uinput.KEY_R, (0,5): uinput.KEY_T, (0,6): uinput.KEY_Y, (0,7): uinput.KEY_U, (0,8): uinput.KEY_I, (0,9): uinput.KEY_O, (0,10): uinput.KEY_P, (0,11): uinput.KEY_BACKSPACE,
    (1,0): uinput.KEY_ESC,  (1,1): uinput.KEY_A, (1,2): uinput.KEY_S, (1,3): uinput.KEY_D, (1,4): uinput.KEY_F, (1,5): uinput.KEY_G, (1,6): uinput.KEY_H, (1,7): uinput.KEY_J, (1,8): uinput.KEY_K, (1,9): uinput.KEY_L, (1,10): uinput.KEY_SEMICOLON, (1,11): uinput.KEY_ENTER,
    (2,0): uinput.KEY_LEFTSHIFT, (2,1): uinput.KEY_Z, (2,2): uinput.KEY_X, (2,3): uinput.KEY_C, (2,4): uinput.KEY_V, (2,5): uinput.KEY_B, (2,6): uinput.KEY_N, (2,7): uinput.KEY_M, (2,8): uinput.KEY_COMMA, (2,9): uinput.KEY_DOT, (2,10): uinput.KEY_UP, (2,11): uinput.KEY_SLASH,
    (3,0): uinput.KEY_LEFTCTRL, (3,1): uinput.KEY_LEFTMETA, (3,2): uinput.KEY_LEFTALT, (3,3): "NUM", (3,4): uinput.KEY_SPACE, (3,5): uinput.KEY_SPACE, (3,6): "SYM", (3,7): uinput.KEY_LEFT, (3,8): uinput.KEY_DOWN, (3,9): uinput.KEY_RIGHT
}

# _NUM Layer
_NUM = {
    (0,1): uinput.KEY_1, (0,2): uinput.KEY_2, (0,3): uinput.KEY_3, (0,4): uinput.KEY_4, (0,5): uinput.KEY_5, (0,6): uinput.KEY_6, (0,7): uinput.KEY_7, (0,8): uinput.KEY_8, (0,9): uinput.KEY_9, (0,10): uinput.KEY_0,
    (1,6): uinput.KEY_LEFT, (1,7): uinput.KEY_DOWN, (1,8): uinput.KEY_UP, (1,9): uinput.KEY_RIGHT
}

# _SYM Layer
_SYM = {
    (0,1): uinput.KEY_LEFTSHIFT, # Placeholder for ! (Shift+1)
    (0,1): uinput.KEY_1, (0,2): uinput.KEY_2, (0,3): uinput.KEY_3, (0,4): uinput.KEY_4, (0,5): uinput.KEY_5, # Will require shift logic for true symbols
    (1,1): uinput.KEY_GRAVE, (1,2): uinput.KEY_1, (1,3): uinput.KEY_LEFTBRACE, (1,4): uinput.KEY_RIGHTBRACE, (1,5): uinput.KEY_BACKSLASH, (1,6): uinput.KEY_MINUS, (1,7): uinput.KEY_EQUAL, (1,8): uinput.KEY_LEFTBRACE, (1,9): uinput.KEY_RIGHTBRACE, (1,10): uinput.KEY_BACKSLASH
}

# 3. INITIALIZE DEVICE
# Collect all possible keys from all maps
all_keys = set()
for layer in [_BASE, _NUM, _SYM]:
    for k in layer.values():
        if isinstance(k, int): all_keys.add(k)

device = uinput.Device(list(all_keys))
time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: GPIO.setup(r, GPIO.OUT, initial=GPIO.HIGH)

# 4. TRACKING
pressed_keys = {} # (r, c): key_sent
active_cols = set()
layers = {"NUM": False, "SYM": False}

def get_key(r, c):
    """Transparency Logic: Checks active layers, falls back to Base"""
    if layers["NUM"] and (r, c) in _NUM: return _NUM[(r, c)]
    if layers["SYM"] and (r, c) in _SYM: return _SYM[(r, c)]
    return _BASE.get((r, c))

print("KramWriter: Full QMK Logic Active.")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            time.sleep(0.0002)
            
            for c_idx, c_pin in enumerate(COLS):
                key_id = (r_idx, c_idx)
                is_down = (GPIO.input(c_pin) == GPIO.LOW)

                # 1. Check for Layer Modifiers (NUM/SYM)
                val = _BASE.get(key_id)
                if val in ["NUM", "SYM"]:
                    layers[val] = is_down
                    continue

                # 2. Key Press
                if is_down and key_id not in pressed_keys:
                    if c_idx not in active_cols:
                        target_key = get_key(r_idx, c_idx)
                        if isinstance(target_key, int):
                            device.emit(target_key, 1)
                            pressed_keys[key_id] = target_key
                            active_cols.add(c_idx)

                # 3. Key Release
                elif not is_down and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0)
                    active_cols.remove(c_idx)
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)

except KeyboardInterrupt:
    GPIO.cleanup()