import RPi.GPIO as GPIO
import time
import uinput

# 1. PHYSICAL CONFIGURATION
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# 2. KEYMAP LAYERS
_BASE = [
    [uinput.KEY_TAB, uinput.KEY_Q, uinput.KEY_W, uinput.KEY_E, uinput.KEY_R, uinput.KEY_T, uinput.KEY_Y, uinput.KEY_U, uinput.KEY_I, uinput.KEY_O, uinput.KEY_P, uinput.KEY_BACKSPACE],
    [uinput.KEY_ESC, uinput.KEY_A, uinput.KEY_S, uinput.KEY_D, uinput.KEY_F, uinput.KEY_G, uinput.KEY_H, uinput.KEY_J, uinput.KEY_K, uinput.KEY_L, uinput.KEY_SEMICOLON, uinput.KEY_ENTER],
    [uinput.KEY_LEFTSHIFT, uinput.KEY_Z, uinput.KEY_X, uinput.KEY_C, uinput.KEY_V, uinput.KEY_B, uinput.KEY_N, uinput.KEY_M, uinput.KEY_COMMA, uinput.KEY_DOT, uinput.KEY_UP, uinput.KEY_SLASH],
    [uinput.KEY_LEFTCTRL, uinput.KEY_LEFTMETA, uinput.KEY_LEFTALT, "NUM", uinput.KEY_SPACE, uinput.KEY_SPACE, "SYM", uinput.KEY_LEFT, uinput.KEY_DOWN, uinput.KEY_RIGHT, uinput.KEY_RESERVED, uinput.KEY_RESERVED]
]

# (I used KEY_RESERVED instead of None to keep the list length safe)
_NUM = [[uinput.KEY_RESERVED]*12 for _ in range(4)]
_NUM[0][1:11] = [uinput.KEY_1, uinput.KEY_2, uinput.KEY_3, uinput.KEY_4, uinput.KEY_5, uinput.KEY_6, uinput.KEY_7, uinput.KEY_8, uinput.KEY_9, uinput.KEY_0]

_SYM = [[uinput.KEY_RESERVED]*12 for _ in range(4)]
_SYM[1][1:11] = [uinput.KEY_GRAVE, uinput.KEY_1, uinput.KEY_LEFTBRACE, uinput.KEY_RIGHTBRACE, uinput.KEY_BACKSLASH, uinput.KEY_MINUS, uinput.KEY_EQUAL, uinput.KEY_LEFTBRACE, uinput.KEY_RIGHTBRACE, uinput.KEY_BACKSLASH]

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

pressed_keys = {} 
active_layers = {"NUM": False, "SYM": False}

print("DIAGNOSTIC MODE STARTING...")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                key_id = (r_idx, c_idx)
                is_down = (GPIO.input(COLS[c_idx]) == GPIO.LOW)
                
                # Check for Layer Modifiers
                val = _BASE[r_idx][c_idx]
                if val == "NUM":
                    active_layers["NUM"] = is_down
                    continue
                if val == "SYM":
                    active_layers["SYM"] = is_down
                    continue

                if is_down and key_id not in pressed_keys:
                    # Determine target
                    target = _BASE[r_idx][c_idx]
                    if active_layers["NUM"] and _NUM[r_idx][c_idx] != uinput.KEY_RESERVED:
                        target = _NUM[r_idx][c_idx]
                    elif active_layers["SYM"] and _SYM[r_idx][c_idx] != uinput.KEY_RESERVED:
                        target = _SYM[r_idx][c_idx]
                    
                    if isinstance(target, int) and target != uinput.KEY_RESERVED:
                        print(f"Sending Press: {target}")
                        device.emit(target, 1)
                        pressed_keys[key_id] = target
                
                elif not is_down and key_id in pressed_keys:
                    print(f"Sending Release: {pressed_keys[key_id]}")
                    device.emit(pressed_keys[key_id], 0)
                    del pressed_keys[key_id]

            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)
except KeyboardInterrupt:
    GPIO.cleanup()