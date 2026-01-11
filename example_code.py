#!/usr/bin/env python3
"""
STABLE version - No flicker, maximum speed
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
import fcntl
import os
import select

# Display setup
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Save terminal settings
old_settings = termios.tcgetattr(sys.stdin)
tty.setraw(sys.stdin.fileno())
old_flags = fcntl.fcntl(sys.stdin, fcntl.F_GETFL)
fcntl.fcntl(sys.stdin, fcntl.F_SETFL, old_flags | os.O_NONBLOCK)

# State tracking
current_state = 1  # 1 = white, 0 = black
space_was_pressed = False

# Start with white screen
display.fill(1)
display.show()

try:
    while True:
        # Check for any input
        ready, _, _ = select.select([sys.stdin], [], [], 0)
        
        if ready:
            # Read ALL available input to check for spacebar state
            space_now_pressed = False
            ctrl_c = False
            
            # Read multiple characters if they're queued
            while True:
                try:
                    char = sys.stdin.read(1)
                    if not char:  # No more data
                        break
                    
                    if char == ' ':
                        space_now_pressed = True
                    elif char == '\x03':  # Ctrl-C
                        ctrl_c = True
                        break
                except (BlockingIOError, OSError):
                    break
            
            if ctrl_c:
                break
            
            # Only update if spacebar state changed
            if space_now_pressed != space_was_pressed:
                space_was_pressed = space_now_pressed
                
                if space_now_pressed:
                    # Space pressed - show black
                    if current_state != 0:
                        display.fill(0)
                        display.show()
                        current_state = 0
                else:
                    # Space released - show white
                    if current_state != 1:
                        display.fill(1)
                        display.show()
                        current_state = 1
        else:
            # No input available - if we thought space was pressed, it's been released
            if space_was_pressed:
                space_was_pressed = False
                if current_state != 1:
                    display.fill(1)
                    display.show()
                    current_state = 1
            
finally:
    # Restore terminal
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
    fcntl.fcntl(sys.stdin, fcntl.F_SETFL, old_flags)
    display.fill(1)
    display.show()