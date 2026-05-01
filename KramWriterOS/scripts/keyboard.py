import RPi.GPIO as GPIO
import time

ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)

for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

print("HARDWARE TEST: Press keys on your matrix...")

try:
    while True:
        for r_idx, r_pin in enumerate(ROWS):
            GPIO.output(r_pin, GPIO.LOW)
            for c_idx, c_pin in enumerate(COLS):
                if GPIO.input(c_pin) == GPIO.LOW:
                    print(f"MATCH FOUND! Row Pin {r_pin} + Col Pin {c_pin}")
                    while GPIO.input(c_pin) == GPIO.LOW:
                        time.sleep(0.1)
            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)
except KeyboardInterrupt:
    GPIO.cleanup()