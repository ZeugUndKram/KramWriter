# minimal_test.py
import time
import RPi.GPIO as GPIO
import spidev

CS_PIN = 8
GPIO.setmode(GPIO.BCM)
GPIO.setup(CS_PIN, GPIO.OUT)

spi = spidev.SpiDev()
spi.open(0, 0)
spi.max_speed_hz = 2000000
spi.mode = 0

# Sharp display initialization sequence
def sharp_cmd(cmd):
    GPIO.output(CS_PIN, GPIO.LOW)
    spi.xfer([cmd])
    GPIO.output(CS_PIN, GPIO.HIGH)

# Clear display
sharp_cmd(0x04)  # Exit shutdown
time.sleep(0.01)
sharp_cmd(0x20)  # Enter extended mode
sharp_cmd(0x80)  # Set VCOM
sharp_cmd(0x01)  # Clear

print(Initialization sent - check display)

spi.close()
GPIO.cleanup()