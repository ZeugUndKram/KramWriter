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

def create_menu_display(selected_idx):
    """Create and display the menu with the selected item in the middle"""
    # Create blank image for drawing.
    # Make sure to create image with mode '1' for 1-bit color.
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    
    # Draw a white background
    draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=WHITE)
    
    # Create the text elements array with font sizes based on position
    # The middle item (index 2 in the displayed array) gets size 50
    # Items 1 and 3 get size 40, items 0 and 4 get size 30
    text_elements = []
    
    # We need to create the display order based on selected index
    # The selected item should be in the middle (position 2 in the 5-element display)
    display_order = []
    for i in range(-2, 3):  # -2, -1, 0, 1, 2
        item_idx = (selected_idx + i) % len(menu_items)
        display_order.append(menu_items[item_idx])
    
    # Assign font sizes based on position in display
    font_sizes = [30, 40, 50, 40, 30]
    
    for i in range(5):
        text_elements.append((display_order[i], font_sizes[i]))
    
    # Calculate total height needed for all text elements
    total_text_height = 0
    spacing = 10  # Spacing between text elements
    all_heights = []
    
    # First pass to calculate heights
    for text, font_size in text_elements:
        font = ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", font_size)
        bbox = font.getbbox(text)
        font_height = bbox[3] - bbox[1]
        all_heights.append(font_height)
        total_text_height += font_height
    
    # Add spacing between elements
    total_text_height += spacing * (len(text_elements) - 1)
    
    # Calculate starting Y position to center all text elements vertically
    current_y = (display.height - total_text_height) // 2
    
    # Draw each text element
    for i, (text, font_size) in enumerate(text_elements):
        font = ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", font_size)
        bbox = font.getbbox(text)
        font_width = bbox[2] - bbox[0]
        
        # Center text horizontally
        x = (display.width - font_width) // 2
        
        # Draw the text
        draw.text((x, current_y), text, font=font, fill=BLACK)
        
        # Move to next position
        current_y += all_heights[i] + spacing
    
    # Display image
    display.image(image)
    display.show()

# Initial display
create_menu_display(selected_index)

print("Menu Navigation Demo")
print("Press UP/DOWN to navigate, 'q' to quit")
print(f"Current selection: {menu_items[selected_index]}")

# Simple keyboard input loop (you'll need to adapt this for your actual input method)
try:
    while True:
        # For testing purposes - you'll need to replace this with actual GPIO button input
        # This assumes you're running on a system with keyboard input
        import sys
        import select
        
        if sys.stdin in select.select([sys.stdin], [], [], 0)[0]:
            key = sys.stdin.read(1)
            
            if key == 'q' or key == 'Q':
                print("Exiting...")
                break
            elif key == 'w' or key == 'W':  # UP
                selected_index = (selected_index - 1) % len(menu_items)
                print(f"Selected: {menu_items[selected_index]}")
                create_menu_display(selected_index)
            elif key == 's' or key == 'S':  # DOWN
                selected_index = (selected_index + 1) % len(menu_items)
                print(f"Selected: {menu_items[selected_index]}")
                create_menu_display(selected_index)
        
        time.sleep(0.1)
        
except KeyboardInterrupt:
    print("\nProgram interrupted")
except Exception as e:
    print(f"Error: {e}")