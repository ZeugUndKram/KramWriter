#!/usr/bin/env python3
"""
Fast input handling script for Sharp Memory Display on Pi Zero 2 W
Spacebar pressed = Black screen, Released = White screen
"""

import board
import busio
import digitalio
from PIL import Image, ImageDraw
import adafruit_sharpmemorydisplay
import time
import pygame

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors - IMPORTANT: Sharp Memory Display uses 1-bit color (0 or 1)
# In the example: BLACK = 0, WHITE = 255 for PIL, but display.fill(1) for white
BLACK = 0
WHITE = 255  # For PIL ImageDraw

def create_solid_image(color):
    """Create a solid color image using PIL, matching the example"""
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), 
                   outline=color, fill=color)
    return image

def main():
    """Main loop using pygame for reliable keyboard input"""
    # Initialize pygame
    pygame.init()
    
    # Set up a minimal pygame display (invisible)
    pygame.display.set_mode((1, 1), pygame.NOFRAME)
    
    print("Starting display controller. Press space to make screen black, release for white.")
    print("Press ESC to exit.")
    
    # Create both images upfront for speed (like caching)
    black_image = create_solid_image(BLACK)
    white_image = create_solid_image(WHITE)
    
    # Start with white screen - using the SAME method as the example
    display.image(white_image)
    display.show()
    print("Screen: WHITE")
    
    current_image = white_image
    
    try:
        while True:
            # Handle pygame events for quit/esc
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    return
                elif event.type == pygame.KEYDOWN:
                    if event.key == pygame.K_ESCAPE:
                        return
            
            # Check if space is pressed
            keys = pygame.key.get_pressed()
            space_pressed = keys[pygame.K_SPACE]
            
            # Determine which image to show
            if space_pressed:
                new_image = black_image
                state_text = "BLACK"
            else:
                new_image = white_image
                state_text = "WHITE"
            
            # Update display if image changed
            if new_image != current_image:
                display.image(new_image)
                display.show()
                current_image = new_image
                print(f"Screen: {state_text}")
            
            # Small delay to prevent CPU hogging
            time.sleep(0.01)  # 10ms - still very responsive
            
    except KeyboardInterrupt:
        pass
    finally:
        # Clean up
        pygame.quit()
        # Clear display to white on exit (using same method)
        display.image(white_image)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()