import RPi.GPIO as GPIO
import time
import uinput

# 1. PINS
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. KEYMAP (Padded to 12 columns)
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

# 3. REGISTER KEYS
all_keys = []
for layer in [_BASE, _NUM]:
    for row in layer:
        for k in row:
            if isinstance(k, int): all_keys.append(k)

device = uinput.Device(list(set(all_keys)))
time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

# Trackers
pressed_keys = {} # (r, c): key_sent
num_layer_active = False

print("KramWriter: Manual Logic Active...")

try:
    while True:
        # Check Layer Modifiers First (Row 3, Col 3 is "NUM")
        # We check the pin directly to avoid logic loops
        GPIO.output(ROWS[3], GPIO.LOW)
        num_layer_active = (GPIO.input(COLS[3]) == GPIO.LOW)
        GPIO.output(ROWS[3], GPIO.HIGH)

        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                
                # Skip the modifier keys themselves to prevent feedback
                if (r_idx, c_idx) == (3, 3): continue 
                
                is_down = (GPIO.input(c_pin) == GPIO.LOW)
                key_id = (r_idx, c_idx)

                if is_down and key_id not in pressed_keys:
                    # Logic: If NUM is held and a NUM key exists, use it. Else use BASE.
                    key = _BASE[r_idx][c_idx]
                    if num_layer_active and _NUM[r_idx][c_idx] is not None:
                        key = _NUM[r_idx][c_idx]
                    
                    if isinstance(key, int):
                        device.emit(key, 1)
                        pressed_keys[key_id] = key
                
                elif not is_down and key_id in pressed_keys:
                    device.emit(pressed_keys[key_id], 0)
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)

except KeyboardInterrupt:
    GPIO.cleanup()