#!/usr/bin/env python3
"""
Simple spacebar input for Sharp Memory Display
Spacebar pressed = Black screen, Released = White screen
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

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors
BLACK = 0
WHITE = 1  # Using 1 like in the example

def setup_nonblocking():
    """Setup terminal for non-blocking input"""
    old_settings = termios.tcgetattr(sys.stdin)
    tty.setcbreak(sys.stdin.fileno())
    return old_settings

def restore_terminal(old_settings):
    """Restore terminal settings"""
    termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)

def main():
    """Main function"""
    print("Spacebar pressed = Black screen, released = White screen")
    print("Press Ctrl-C to exit")
    
    # Setup non-blocking input
    old_settings = setup_nonblocking()
    
    # Start with white screen
    display.fill(WHITE)
    display.show()
    print("Screen: WHITE")
    
    last_state = WHITE
    
    try:
        while True:
            # Check for any input
            ready, _, _ = select.select([sys.stdin], [], [], 0)
            
            if ready:
                # Read the key
                key = sys.stdin.read(1)
                
                if key == ' ':  # Space pressed
                    new_state = BLACK
                elif key == '\x03':  # Ctrl-C
                    break
                else:
                    new_state = WHITE
            else:
                # No key pressed
                new_state = WHITE
            
            # Update display if state changed
            if new_state != last_state:
                display.fill(new_state)
                display.show()
                last_state = new_state
                print(f"Screen: {'BLACK' if new_state == BLACK else 'WHITE'}")
            
            # Tiny delay
            time.sleep(0.001)
            
    except KeyboardInterrupt:
        pass
    finally:
        # Cleanup
        restore_terminal(old_settings)
        display.fill(WHITE)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()