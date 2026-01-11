#!/usr/bin/env python3
"""
Fast input handling script for Sharp Memory Display on Pi Zero 2 W
Spacebar pressed = Black screen, Released = White screen
"""

import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
import time
import keyboard  # pip install keyboard

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors
BLACK = 0
WHITE = 255

def main():
    """Main loop using keyboard library for input"""
    print("Starting display controller. Press space to make screen black, release for white.")
    print("Press Ctrl-C to exit.")
    
    last_state = None
    
    try:
        while True:
            # Check if space is pressed
            space_pressed = keyboard.is_pressed('space')
            
            # Determine display state
            if space_pressed:
                new_state = BLACK
                state_text = "BLACK"
            else:
                new_state = WHITE
                state_text = "WHITE"
            
            # Update display if state changed
            if new_state != last_state:
                display.fill(new_state)
                display.show()
                last_state = new_state
                print(f"Screen: {state_text}")
            
            # Small delay
            time.sleep(0.01)  # 10ms
            
    except KeyboardInterrupt:
        pass
    finally:
        # Clear display to white on exit
        display.fill(WHITE)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()