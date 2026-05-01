import RPi.GPIO as GPIO
import time
import uinput

# 1. PHYSICAL CONFIGURATION
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. KEYMAP LAYERS (Each row MUST have 12 entries)
_BASE = [
    [uinput.KEY_TAB, uinput.KEY_Q, uinput.KEY_W, uinput.KEY_E, uinput.KEY_R, uinput.KEY_T, uinput.KEY_Y, uinput.KEY_U, uinput.KEY_I, uinput.KEY_O, uinput.KEY_P, uinput.KEY_BACKSPACE],
    [uinput.KEY_ESC, uinput.KEY_A, uinput.KEY_S, uinput.KEY_D, uinput.KEY_F, uinput.KEY_G, uinput.KEY_H, uinput.KEY_J, uinput.KEY_K, uinput.KEY_L, uinput.KEY_SEMICOLON, uinput.KEY_ENTER],
    [uinput.KEY_LEFTSHIFT, uinput.KEY_Z, uinput.KEY_X, uinput.KEY_C, uinput.KEY_V, uinput.KEY_B, uinput.KEY_N, uinput.KEY_M, uinput.KEY_COMMA, uinput.KEY_DOT, uinput.KEY_UP, uinput.KEY_SLASH],
    [uinput.KEY_LEFTCTRL, uinput.KEY_LEFTMETA, uinput.KEY_LEFTALT, "NUM", uinput.KEY_SPACE, uinput.KEY_SPACE, "SYM", uinput.KEY_LEFT, uinput.KEY_DOWN, uinput.KEY_RIGHT, None, None]
]

_NUM = [
    [None, uinput.KEY_1, uinput.KEY_2, uinput.KEY_3, uinput.KEY_4, uinput.KEY_5, uinput.KEY_6, uinput.KEY_7, uinput.KEY_8, uinput.KEY_9, uinput.KEY_0, None],
    [None, None, None, None, None, None, uinput.KEY_LEFT, uinput.KEY_DOWN, uinput.KEY_UP, uinput.KEY_RIGHT, None, None],
    [None, None, None, None, None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None, None, None, None, None]
]

_SYM = [
    [None, uinput.KEY_KPSLASH, uinput.KEY_KP7, uinput.KEY_KP8, uinput.KEY_KP9, uinput.KEY_KPMINUS, None, None, None, None, None, None],
    [None, uinput.KEY_GRAVE, uinput.KEY_KPDOT, uinput.KEY_LEFTBRACE, uinput.KEY_RIGHTBRACE, uinput.KEY_BACKSLASH, uinput.KEY_MINUS, uinput.KEY_EQUAL, uinput.KEY_LEFTBRACE, uinput.KEY_RIGHTBRACE, uinput.KEY_BACKSLASH, None],
    [None, None, None, None, None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None, None, None, None, None]
]

# 3. INITIALIZATION
all_keys = set()
for layer in [_BASE, _NUM, _SYM]:
    for row in layer:
        for k in row:
            if isinstance(k, int): all_keys.add(k)

device = uinput.Device(list(all_keys))
time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

pressed_keys = {} # Store {(row, col): key_sent}
active_layers = {"NUM": False, "SYM": False}

def get_target_key(r, c):
    """Safely finds the key for the current row/col and active layer"""
    try:
        if active_layers["NUM"] and _NUM[r][c] is not None:
            return _NUM[r][c]
        if active_layers["SYM"] and _SYM[r][c] is not None:
            return _SYM[r][c]
        return _BASE[r][c]
    except (IndexError, TypeError):
        return None

print("KramWriter Driver ACTIVE.")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                key_id = (r_idx, c_idx)
                is_physically_down = (GPIO.input(COLS[c_idx]) == GPIO.LOW)
                
                # Check for Layer Modifiers
                base_val = _BASE[r_idx][c_idx]
                if base_val in ["NUM", "SYM"]:
                    active_layers[base_val] = is_physically_down
                    continue

                if is_physically_down and key_id not in pressed_keys:
                    target_key = get_target_key(r_idx, c_idx)
                    if isinstance(target_key, int):
                        device.emit(target_key, 1)
                        pressed_keys[key_id] = target_key
                
                elif not is_physically_down and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0)
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)

except KeyboardInterrupt:
    GPIO.cleanup()