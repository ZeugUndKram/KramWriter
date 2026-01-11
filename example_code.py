#!/usr/bin/env python3
"""
Fast input handling script for Sharp Memory Display on Pi Zero 2 W
Spacebar pressed = Black screen, Released = White screen
"""

import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
import select
import sys
import termios
import tty
import time
import os

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors
BLACK = 0
WHITE = 255

def setup_nonblocking_input():
    """Configure stdin for non-blocking raw input"""
    old_settings = termios.tcgetattr(sys.stdin)
    tty.setraw(sys.stdin.fileno())
    return old_settings

def restore_input(old_settings):
    """Restore terminal settings"""
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)

def check_space_pressed():
    """Fast non-blocking check for spacebar press using select()"""
    # Check if there's input available
    ready, _, _ = select.select([sys.stdin], [], [], 0)
    if ready:
        # Read available input
        char = sys.stdin.read(1)
        if char == ' ':
            return True
        elif char == '\x03':  # Ctrl-C
            return None
    return False

def main():
    """Main loop with optimized input handling"""
    print("Starting display controller. Press space to make screen black, release for white.")
    print("Press Ctrl-C to exit.")
    
    # Setup non-blocking input
    old_settings = setup_nonblocking_input()
    
    try:
        last_state = None  # Track last display state to avoid unnecessary updates
        
        while True:
            # Fast input check
            space_state = check_space_pressed()
            
            if space_state is None:  # Ctrl-C detected
                break
            
            # Determine new display state
            if space_state:  # Space pressed = black screen
                new_state = BLACK
            else:  # Space not pressed = white screen
                new_state = WHITE
            
            # Only update display if state changed
            if new_state != last_state:
                display.fill(new_state)
                display.show()
                last_state = new_state
                
                # Debug output (optional)
                # print(f"Screen: {'BLACK' if new_state == BLACK else 'WHITE'}")
            
            # Small sleep to prevent CPU hogging while still being responsive
            # Adjust based on needed responsiveness vs CPU usage
            time.sleep(0.001)  # 1ms delay - very responsive
            
    except KeyboardInterrupt:
        pass
    finally:
        # Clean up
        restore_input(old_settings)
        # Clear display to white on exit
        display.fill(WHITE)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    # Set high priority for the process (requires sudo)
    try:
        os.nice(-10)  # Increase priority
    except:
        pass  # Ignore if not running as sudo
    
    main()