#!/usr/bin/env python3
"""
PROPER STATE TRACKING - No flicker
Spacebar pressed = Black screen, released = White screen
400x240 Sharp Memory Display
"""

import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
import sys
import termios
import tty
import select
import time

# Display setup
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Save terminal
old_settings = termios.tcgetattr(sys.stdin)
tty.setcbreak(sys.stdin.fileno())  # Use cbreak instead of raw for better handling

# State
display_state = 1  # 1=white, 0=black
key_state = 0      # 0=no key, 32=space, 3=ctrl-c

# Start with white
display.fill(1)
display.show()

print("Running. Space=Black, Release=White, Ctrl-C to exit")

try:
    while True:
        # Check for keypress
        ready, _, _ = select.select([sys.stdin], [], [], 0.01)  # 10ms timeout
        
        if ready:
            # Read key
            try:
                key = ord(sys.stdin.read(1))
            except TypeError:
                continue
            
            if key == 3:  # Ctrl-C
                break
            elif key == 32:  # Space
                if key_state != 32:  # Space newly pressed
                    key_state = 32
                    if display_state != 0:
                        display.fill(0)
                        display.show()
                        display_state = 0
            else:
                # Any other key means space is released
                if key_state == 32:  # Space was pressed, now released
                    key_state = 0
                    if display_state != 1:
                        display.fill(1)
                        display.show()
                        display_state = 1
        else:
            # No key pressed
            if key_state == 32:  # Space was pressed but no input now
                key_state = 0
                if display_state != 1:
                    display.fill(1)
                    display.show()
                    display_state = 1
            
finally:
    # Cleanup
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
    display.fill(1)
    display.show()
    print("\nExited")