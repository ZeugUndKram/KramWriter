# SPDX-FileCopyrightText: 2021 ladyada for Adafruit Industries
# SPDX-License-Identifier: MIT

"""
This demo will fill the screen with white, draw a black box on top
and then print Hello World! in the center of the display

This example is for use on (Linux) computers that are using CPython with
Adafruit Blinka to support CircuitPython libraries. CircuitPython does
not support PIL/pillow (python imaging library)!
"""

import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import time
import sys
import os

import adafruit_sharpmemorydisplay

# Colors
BLACK = 0
WHITE = 255

# Parameters to Change
BORDER = 5

spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)  # inverted chip select

# display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 96, 96)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)
#display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Clear display.
display.fill(1)
display.show()

# Define menu items and their base font sizes (when selected = middle position)
menu_items = ["Write", "Anki", "Zeugtris", "Settings", "Credits"]
selected_index = 2  # Start with Zeugtris in the middle (index 2)

# PRELOAD ALL FONTS - DO THIS ONLY ONCE
print("Loading fonts...")
font_cache = {
    30: ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", 30),
    40: ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", 40),
    50: ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", 50)
}

# Pre-calculate font metrics for each size
font_metrics = {}
for size, font in font_cache.items():
    # Get metrics for all menu items with this font size
    metrics = {}
    for item in menu_items:
        bbox = font.getbbox(item)
        metrics[item] = {
            'width': bbox[2] - bbox[0],
            'height': bbox[3] - bbox[1]
        }
    font_metrics[size] = metrics

print("Fonts loaded and cached!")

# Create image and draw objects ONCE and reuse them
image = Image.new("1", (display.width, display.height))
draw = ImageDraw.Draw(image)

def create_menu_display(selected_idx):
    """Create and display the menu with the selected item in the middle"""
    # Clear the image (fill with white)
    draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=WHITE)
    
    # Create the display order based on selected index
    # The selected item should be in the middle (position 2 in the 5-element display)
    display_order = []
    for i in range(-2, 3):  # -2, -1, 0, 1, 2
        item_idx = (selected_idx + i) % len(menu_items)
        display_order.append(menu_items[item_idx])
    
    # Assign font sizes based on position in display
    font_sizes = [30, 40, 50, 40, 30]
    
    # Calculate total height needed for all text elements
    total_text_height = 0
    spacing = 10  # Spacing between text elements
    
    for i in range(5):
        text = display_order[i]
        font_size = font_sizes[i]
        total_text_height += font_metrics[font_size][text]['height']
    
    # Add spacing between elements
    total_text_height += spacing * 4
    
    # Calculate starting Y position to center all text elements vertically
    current_y = (display.height - total_text_height) // 2
    
    # Draw each text element
    for i in range(5):
        text = display_order[i]
        font_size = font_sizes[i]
        font = font_cache[font_size]
        
        # Get pre-calculated width
        text_width = font_metrics[font_size][text]['width']
        
        # Center text horizontally
        x = (display.width - text_width) // 2
        
        # Draw the text
        draw.text((x, current_y), text, font=font, fill=BLACK)
        
        # Move to next position
        current_y += font_metrics[font_size][text]['height'] + spacing
    
    # Display image
    display.image(image)
    display.show()

# Initial display
create_menu_display(selected_index)

print("Menu Navigation Demo")
print("Press UP/DOWN arrow keys to navigate, 'q' to quit")
print(f"Current selection: {menu_items[selected_index]}")

# Simple input for testing - MUCH faster now
try:
    import termios
    import tty
    
    # Save terminal settings
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    
    # Set terminal to raw mode for single key reading
    tty.setraw(fd)
    
    try:
        last_update_time = time.time()
        update_count = 0
        
        while True:
            # Check for key press without blocking for too long
            import select
            if select.select([sys.stdin], [], [], 0.01)[0]:
                key = sys.stdin.read(1)
                
                if key == 'q' or key == 'Q':
                    print("\nExiting...")
                    break
                elif key == '\x1b':  # Escape sequence for arrow keys
                    # Read next two characters
                    next1 = sys.stdin.read(1)
                    next2 = sys.stdin.read(1)
                    if next1 == '[':
                        if next2 == 'A':  # UP arrow
                            selected_index = (selected_index - 1) % len(menu_items)
                        elif next2 == 'B':  # DOWN arrow
                            selected_index = (selected_index + 1) % len(menu_items)
                        
                        update_count += 1
                        if update_count % 10 == 0:  # Print every 10 updates
                            print(f"\rUpdates: {update_count}, Selected: {menu_items[selected_index]:<15}", end="", flush=True)
                        
                        create_menu_display(selected_index)
                
                # Also accept w/s keys
                elif key == 'w' or key == 'W':
                    selected_index = (selected_index - 1) % len(menu_items)
                    create_menu_display(selected_index)
                elif key == 's' or key == 'S':
                    selected_index = (selected_index + 1) % len(menu_items)
                    create_menu_display(selected_index)
            
            # Optional: Add a small delay to prevent CPU hogging
            time.sleep(0.001)
    
    finally:
        # Restore terminal settings
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        print(f"\nTotal updates: {update_count}")
        print("Terminal restored.")

except ImportError:
    # Fallback for systems without termios
    print("Termios not available. Using simple input.")
    
    while True:
        try:
            user_input = input("Press w/s/q then Enter: ").strip().lower()
            
            if user_input == 'q':
                print("Exiting...")
                break
            elif user_input == 'w':
                selected_index = (selected_index - 1) % len(menu_items)
                print(f"Selected: {menu_items[selected_index]}")
                create_menu_display(selected_index)
            elif user_input == 's':
                selected_index = (selected_index + 1) % len(menu_items)
                print(f"Selected: {menu_items[selected_index]}")
                create_menu_display(selected_index)
        
        except KeyboardInterrupt:
            print("\nExiting...")
            break
except KeyboardInterrupt:
    print("\nProgram interrupted")
except Exception as e:
    print(f"Error: {e}")