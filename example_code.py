#!/usr/bin/env python3
"""
Spacebar pressed = Black screen
Spacebar released = White screen
For 400x240 Sharp Memory Display
"""

import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
import time
import keyboard  # pip install keyboard

# Initialize display for 400x240
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def main():
    """Simple main loop"""
    print(f"Display size: {display.width}x{display.height}")
    print("Spacebar pressed = Black screen")
    print("Spacebar released = White screen")
    print("Press Ctrl-C to exit.")
    
    # Start with white screen
    display.fill(1)  # 1 = white
    display.show()
    print("Screen: WHITE")
    
    current_state = 1  # 1 = white, 0 = black
    
    try:
        while True:
            # Check if space is pressed
            space_pressed = keyboard.is_pressed('space')
            
            # Determine screen color
            if space_pressed:
                new_state = 0  # black
                state_text = "BLACK"
            else:
                new_state = 1  # white
                state_text = "WHITE"
            
            # Update display if state changed
            if new_state != current_state:
                display.fill(new_state)
                display.show()
                current_state = new_state
                print(f"Screen: {state_text}")
            
            # Small delay
            time.sleep(0.01)
            
    except KeyboardInterrupt:
        pass
    finally:
        # Clear to white on exit
        display.fill(1)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()