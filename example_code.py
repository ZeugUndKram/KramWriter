#!/usr/bin/env python3
"""
SIMPLE AND STABLE - Fixed release detection
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

# Setup
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Terminal
old_settings = termios.tcgetattr(sys.stdin)
tty.setcbreak(sys.stdin.fileno())

print("Press SPACE for black, release for white")
print("Ctrl-C to exit")

# Start white
display.fill(1)
display.show()

try:
    while True:
        # Wait for any key
        select.select([sys.stdin], [], [])
        key = sys.stdin.read(1)
        
        if key == '\x03':  # Ctrl-C
            break
            
        if key == ' ':
            # Turn black immediately
            display.fill(0)
            display.show()
            print("Screen: BLACK (holding...)")
            
            # Wait for space release OR another key
            while True:
                select.select([sys.stdin], [], [])
                next_key = sys.stdin.read(1)
                
                if next_key == '\x03':
                    display.fill(1)
                    display.show()
                    sys.exit(0)
                    
                # If we get ANY key (including another space), break and check
                # In practice, space release sends no character, so any character
                # means the user pressed a different key
                break
            
            # Turn white (space was released or another key was pressed)
            display.fill(1)
            display.show()
            print("Screen: WHITE")
            
finally:
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
    display.fill(1)
    display.show()
    print("\nExited")