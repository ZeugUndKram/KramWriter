import RPi.GPIO as GPIO
import time

# Defined from your setup
ROWS = [26, 8, 22, 24]
COLS = [2, 3, 4, 14, 15, 18, 11, 19, 13, 5, 16, 20, 21]

GPIO.setmode(GPIO.BCM)

# Set columns as inputs with pull-up resistors
for c in COLS:
    GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)

# Set rows as outputs, default to High
for r in ROWS:
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

print("Mapping mode active. Press a key on your matrix...")

try:
    while True:
        for r_pin in ROWS:
            GPIO.output(r_pin, GPIO.LOW) # Trigger the row
            for c_pin in COLS:
                if GPIO.input(c_pin) == GPIO.LOW:
                    print(f"Key Pressed! Row GPIO: {r_pin} | Col GPIO: {c_pin}")
                    while GPIO.input(c_pin) == GPIO.LOW: # Debounce
                        time.sleep(0.1)
            GPIO.output(r_pin, GPIO.HIGH) # Reset the row
            time.sleep(0.01)
except KeyboardInterrupt:
    GPIO.cleanup()