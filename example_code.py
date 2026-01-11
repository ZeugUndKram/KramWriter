#!/usr/bin/env python3
"""
Debug version - follows the exact pattern from the example
"""

import board
import busio
import digitalio
from PIL import Image, ImageDraw
import adafruit_sharpmemorydisplay
import sys
import termios
import tty
import time

# Initialize exactly like the example
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors
BLACK = 0
WHITE = 255  # Note: Using 255 for PIL, but 1 for display.fill()

def create_solid_image(color):
    """Create a solid color image"""
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), 
                   outline=color, fill=color)
    return image

def main():
    """Main function using PIL images like the example"""
    print("Spacebar = Black, Release = White")
    print("Press Ctrl-C to exit")
    
    # Save terminal settings
    old_settings = termios.tcgetattr(sys.stdin)
    
    try:
        # Set terminal to non-blocking
        tty.setcbreak(sys.stdin.fileno())
        
        # Create images upfront
        black_image = create_solid_image(BLACK)
        white_image = create_solid_image(WHITE)
        
        # Start with white screen (like example)
        display.image(white_image)
        display.show()
        print("Screen: WHITE")
        
        current_image = white_image
        space_pressed = False
        
        while True:
            # Check for input
            import select
            ready, _, _ = select.select([sys.stdin], [], [], 0)
            
            if ready:
                key = sys.stdin.read(1)
                if key == ' ':
                    space_pressed = True
                elif key == '\x03':  # Ctrl-C
                    break
                else:
                    space_pressed = False
            else:
                space_pressed = False
            
            # Determine which image to show
            if space_pressed:
                new_image = black_image
                state_text = "BLACK"
            else:
                new_image = white_image
                state_text = "WHITE"
            
            # Update if changed
            if new_image != current_image:
                display.image(new_image)
                display.show()
                current_image = new_image
                print(f"Screen: {state_text}")
            
            time.sleep(0.01)
            
    except KeyboardInterrupt:
        pass
    finally:
        # Restore terminal
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
        # Clear to white
        display.image(white_image)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()