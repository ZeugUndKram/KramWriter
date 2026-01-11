#!/usr/bin/env python3
"""
DEBOUNCED - No flicker, simple and reliable
"""

import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
import sys
import termios
import tty
import select

# Display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Terminal setup
old_settings = termios.tcgetattr(sys.stdin)
tty.setcbreak(sys.stdin.fileno())

# State
screen_is_black = False

# Start white
display.fill(1)
display.show()

try:
    while True:
        # Wait for key event (blocking until keypress)
        select.select([sys.stdin], [], [])
        
        # Read the key
        key = sys.stdin.read(1)
        
        if key == '\x03':  # Ctrl-C
            break
        elif key == ' ':
            # Space pressed - turn black
            if not screen_is_black:
                display.fill(0)
                display.show()
                screen_is_black = True
            
            # Wait for space release
            while True:
                select.select([sys.stdin], [], [])
                release_key = sys.stdin.read(1)
                if release_key == '\x03':
                    display.fill(1)
                    display.show()
                    sys.exit(0)
                elif release_key != ' ':  # Any non-space key means space released
                    break
            
            # Space released - turn white
            if screen_is_black:
                display.fill(1)
                display.show()
                screen_is_black = False
                
finally:
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
    display.fill(1)
    display.show()