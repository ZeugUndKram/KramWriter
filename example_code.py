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
import pygame

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors
BLACK = 0
WHITE = 255

def main():
    """Main loop using pygame for reliable keyboard input"""
    # Initialize pygame
    pygame.init()
    
    # Set up a minimal pygame display (invisible)
    pygame.display.set_mode((1, 1), pygame.NOFRAME)
    
    print("Starting display controller. Press space to make screen black, release for white.")
    print("Press ESC to exit.")
    
    last_state = None
    
    try:
        while True:
            # Handle pygame events
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    return
                elif event.type == pygame.KEYDOWN:
                    if event.key == pygame.K_ESCAPE:
                        return
            
            # Check if space is pressed
            keys = pygame.key.get_pressed()
            space_pressed = keys[pygame.K_SPACE]
            
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
            
            # Small delay to prevent CPU hogging
            time.sleep(0.01)  # 10ms - still very responsive
            
    except KeyboardInterrupt:
        pass
    finally:
        # Clean up
        pygame.quit()
        # Clear display to white on exit
        display.fill(WHITE)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()