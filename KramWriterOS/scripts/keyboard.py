import RPi.GPIO as GPIO
import time
import uinput

# PHYSICAL CONNECTIONS
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# LAYER DEFINITIONS
_BASE = [
    [uinput.KEY_TAB, uinput.KEY_Q, uinput.KEY_W, uinput.KEY_E, uinput.KEY_R, uinput.KEY_T, uinput.KEY_Y, uinput.KEY_U, uinput.KEY_I, uinput.KEY_O, uinput.KEY_P, uinput.KEY_BACKSPACE],
    [uinput.KEY_ESC, uinput.KEY_A, uinput.KEY_S, uinput.KEY_D, uinput.KEY_F, uinput.KEY_G, uinput.KEY_H, uinput.KEY_J, uinput.KEY_K, uinput.KEY_L, uinput.KEY_SEMICOLON, uinput.KEY_ENTER],
    [uinput.KEY_LEFTSHIFT, uinput.KEY_Z, uinput.KEY_X, uinput.KEY_C, uinput.KEY_V, uinput.KEY_B, uinput.KEY_N, uinput.KEY_M, uinput.KEY_COMMA, uinput.KEY_DOT, uinput.KEY_UP, uinput.KEY_SLASH],
    [uinput.KEY_LEFTCTRL, uinput.KEY_LEFTMETA, uinput.KEY_LEFTALT, "NUM", uinput.KEY_SPACE, uinput.KEY_SPACE, "SYM", uinput.KEY_LEFT, uinput.KEY_DOWN, uinput.KEY_RIGHT]
]

_NUM = [
    [None, uinput.KEY_1, uinput.KEY_2, uinput.KEY_3, uinput.KEY_4, uinput.KEY_5, uinput.KEY_6, uinput.KEY_7, uinput.KEY_8, uinput.KEY_9, uinput.KEY_0, None],
    [None, None, None, None, None, None, uinput.KEY_LEFT, uinput.KEY_DOWN, uinput.KEY_UP, uinput.KEY_RIGHT, None, None],
    [None, None, None, None, None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None, None, None]
]

_SYM = [
    [None, uinput.KEY_1, uinput.KEY_2, uinput.KEY_3, uinput.KEY_4, uinput.KEY_5, uinput.KEY_6, uinput.KEY_7, uinput.KEY_8, uinput.KEY_9, uinput.KEY_0, None], # Adjusted for standard shifted symbols
    [None, uinput.KEY_GRAVE, uinput.KEY_GRAVE, uinput.KEY_LEFTBRACE, uinput.KEY_RIGHTBRACE, uinput.KEY_BACKSLASH, uinput.KEY_MINUS, uinput.KEY_EQUAL, uinput.KEY_LEFTBRACE, uinput.KEY_RIGHTBRACE, uinput.KEY_BACKSLASH, None],
    [None, None, None, None, None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None, None, None]
]

# INITIALIZE DEVICE (Registering all used keys)
all_keys = [k for row in _BASE for k in row if isinstance(k, int)] + \
           [k for row in _NUM for k in row if isinstance(k, int)] + \
           [k for row in _SYM for k in row if isinstance(k, int)] + \
           [uinput.KEY_LEFTSHIFT, uinput.KEY_RESETSYS] # Buffers

device = uinput.Device(list(set(all_keys)))

GPIO.setmode(GPIO.BCM)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

current_layer = _BASE
pressed_keys = {}

def get_key(r, c):
    # Determine which layer to pull from
    layer = _BASE
    if pressed_keys.get("NUM"): layer = _NUM
    elif pressed_keys.get("SYM"): layer = _SYM
    
    try:
        return layer[r][c] if layer[r][c] is not None else _BASE[r][c]
    except IndexError:
        return None

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                state = GPIO.input(c_pin)
                key_id = (r_idx, c_idx)
                
                if state == GPIO.LOW and key_id not in pressed_keys:
                    val = _BASE[r_idx][c_idx]
                    if val == "NUM": pressed_keys["NUM"] = True
                    elif val == "SYM": pressed_keys["SYM"] = True
                    else:
                        actual_key = get_key(r_idx, c_idx)
                        if actual_key:
                            device.emit(actual_key, 1)
                            pressed_keys[key_id] = actual_key
                            
                elif state == GPIO.HIGH and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0)
                    del pressed_keys[key_id]
                
                elif state == GPIO.HIGH and (r_idx, c_idx) == (3, 3): # NUM release
                    pressed_keys.pop("NUM", None)
                elif state == GPIO.HIGH and (r_idx, c_idx) == (3, 6): # SYM release
                    pressed_keys.pop("SYM", None)

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)
except KeyboardInterrupt:
    GPIO.cleanup()