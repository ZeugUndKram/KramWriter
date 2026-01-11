#!/usr/bin/env python3
"""
MINIMAL MAX SPEED - No prints, no overhead
"""

import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
import sys
import termios
import tty
import fcntl
import os
import select

# Display setup
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Save terminal
old_settings = termios.tcgetattr(sys.stdin)
tty.setraw(sys.stdin.fileno())
old_flags = fcntl.fcntl(sys.stdin, fcntl.F_GETFL)
fcntl.fcntl(sys.stdin, fcntl.F_SETFL, old_flags | os.O_NONBLOCK)

# State
state = 1
display.fill(1)
display.show()

try:
    while True:
        # Check for input (FASTEST POSSIBLE)
        ready, _, _ = select.select([sys.stdin], [], [], 0)
        if ready:
            char = sys.stdin.read(1)
            if char == '\x03':
                break
            elif char == ' ':
                # Consume any buffered input
                while select.select([sys.stdin], [], [], 0)[0]:
                    sys.stdin.read(1)
                if state != 0:
                    display.fill(0)
                    display.show()
                    state = 0
                continue
        
        # Space not pressed
        if state != 1:
            display.fill(1)
            display.show()
            state = 1
            
finally:
    # Restore
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
    fcntl.fcntl(sys.stdin, fcntl.F_SETFL, old_flags)
    display.fill(1)
    display.show()