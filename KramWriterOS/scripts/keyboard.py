import RPi.GPIO as GPIO
import time
import uinput

# Basic Pins
ROWS = [26, 8, 22, 24]
COLS = [13, 5, 19, 16, 20, 21, 2, 3, 4, 14, 15, 18]

# Create device with just ONE key for testing (the letter A)
device = uinput.Device([uinput.KEY_A])
time.sleep(1)

GPIO.setmode(GPIO.BCM)
GPIO.setwarnings(False)
for c in COLS: GPIO.setup(c, GPIO.IN, pull_up_down=GPIO.PUD_UP)
for r in ROWS: 
    GPIO.setup(r, GPIO.OUT)
    GPIO.output(r, GPIO.HIGH)

print("EMERGENCY TEST: Every key press should type 'a'...")

try:
    while True:
        for r_pin in ROWS:
            GPIO.output(r_pin, GPIO.LOW)
            for c_pin in COLS:
                if GPIO.input(c_pin) == GPIO.LOW:
                    print(f"Physical Hit! Row {r_pin} Col {c_pin}")
                    device.emit(uinput.KEY_A, 1) # Press A
                    device.emit(uinput.KEY_A, 0) # Release A
                    while GPIO.input(c_pin) == GPIO.LOW:
                        time.sleep(0.05)
            GPIO.output(r_pin, GPIO.HIGH)
        time.sleep(0.01)
except KeyboardInterrupt:
    GPIO.cleanup()