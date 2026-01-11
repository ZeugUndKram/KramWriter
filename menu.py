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

import adafruit_sharpmemorydisplay

# Colors
BLACK = 0
WHITE = 255

spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)  # inverted chip select

display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Clear display once at startup
display.fill(1)
display.show()

# Define menu items
menu_items = ["Write", "Anki", "Zeugtris", "Settings", "Credits"]
selected_index = 2  # Start with Zeugtris in the middle

# PRELOAD AND CACHE EVERYTHING
font_cache = {
    30: ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", 30),
    40: ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", 40),
    50: ImageFont.truetype("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf", 50)
}

# Cache rendered text as bitmaps
text_bitmaps = {}
for size, font in font_cache.items():
    text_bitmaps[size] = {}
    for item in menu_items:
        bbox = font.getbbox(item)
        width = bbox[2] - bbox[0]
        height = bbox[3] - bbox[1]
        
        # Create bitmap for this text
        img = Image.new("1", (width, height), WHITE)
        draw_temp = ImageDraw.Draw(img)
        draw_temp.text((0, 0), item, font=font, fill=BLACK)
        text_bitmaps[size][item] = {
            'image': img,
            'width': width,
            'height': height
        }

# Create main image and draw objects once
image = Image.new("1", (display.width, display.height), WHITE)
draw = ImageDraw.Draw(image)

# Store previous state for partial updates
prev_display_order = []
prev_selected_index = None

# Track dirty regions for partial redraw
dirty_regions = []

def compute_display_order(selected_idx):
    """Compute display indices instead of strings"""
    indices = []
    for i in range(-2, 3):
        indices.append((selected_idx + i) % len(menu_items))
    return indices

def get_text_position(item_idx, display_pos, total_height, start_y, spacing):
    """Calculate position for a text item"""
    size = [30, 40, 50, 40, 30][display_pos]
    metrics = text_bitmaps[size][menu_items[item_idx]]
    
    # Calculate Y position
    y = start_y
    for pos in range(display_pos):
        prev_size = [30, 40, 50, 40, 30][pos]
        prev_idx = (selected_index + pos - 2) % len(menu_items)
        y += text_bitmaps[prev_size][menu_items[prev_idx]]['height'] + spacing
    
    # Center horizontally
    x = (display.width - metrics['width']) // 2
    
    return x, y, size, metrics

def create_menu_display(selected_idx):
    """Create and display the menu with the selected item in the middle"""
    global prev_display_order, prev_selected_index
    
    # Get current display order as indices
    display_indices = compute_display_order(selected_idx)
    
    # Skip redraw if nothing changed
    if prev_selected_index == selected_idx:
        return
    
    # Calculate total height and starting position
    total_height = 0
    spacing = 10
    
    for pos in range(5):
        item_idx = display_indices[pos]
        size = [30, 40, 50, 40, 30][pos]
        total_height += text_bitmaps[size][menu_items[item_idx]]['height']
    
    total_height += spacing * 4
    start_y = (display.height - total_height) // 2
    
    # If we have previous state, do partial update
    if prev_display_order and prev_selected_index is not None:
        # Clear only the regions that need updating
        for pos in range(5):
            prev_idx = prev_display_order[pos]
            curr_idx = display_indices[pos]
            
            # If item changed at this position, mark region as dirty
            if prev_idx != curr_idx or prev_selected_index != selected_idx:
                x, y, size, metrics = get_text_position(
                    prev_idx, pos, 
                    total_height, start_y, spacing
                )
                # Clear old text region (extend slightly to ensure full clear)
                draw.rectangle(
                    [x-1, y-1, x + metrics['width'] + 1, y + metrics['height'] + 1],
                    fill=WHITE
                )
                
                # Draw new text if we have it
                if curr_idx != prev_idx:
                    x_new, y_new, size_new, metrics_new = get_text_position(
                        curr_idx, pos, 
                        total_height, start_y, spacing
                    )
                    image.paste(
                        text_bitmaps[size_new][menu_items[curr_idx]]['image'],
                        (x_new, y_new)
                    )
                else:
                    # Same text, just redraw it
                    image.paste(
                        text_bitmaps[size][menu_items[curr_idx]]['image'],
                        (x, y)
                    )
    else:
        # Initial draw or full redraw
        draw.rectangle([0, 0, display.width, display.height], fill=WHITE)
        
        current_y = start_y
        for pos in range(5):
            item_idx = display_indices[pos]
            size = [30, 40, 50, 40, 30][pos]
            metrics = text_bitmaps[size][menu_items[item_idx]]
            
            x = (display.width - metrics['width']) // 2
            image.paste(metrics['image'], (x, current_y))
            current_y += metrics['height'] + spacing
    
    # Update display
    display.image(image)
    display.show()
    
    # Store current state for next comparison
    prev_display_order = display_indices
    prev_selected_index = selected_idx

# Initial display
create_menu_display(selected_index)

print("Ready. Use arrow keys to navigate, 'q' to quit")

# Optimized input loop
try:
    import termios
    import tty
    import select
    
    # Save terminal settings
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    
    # Set terminal to raw mode
    tty.setraw(fd)
    
    last_key_time = 0
    key_debounce = 0.05  # 50ms debounce
    
    try:
        while True:
            current_time = time.time()
            
            # Check for input with slightly longer timeout
            if select.select([sys.stdin], [], [], 0.05)[0]:
                key = sys.stdin.read(1)
                
                if key == 'q' or key == 'Q':
                    break
                elif key == '\x1b':  # Escape sequence
                    # Read next characters with timeout
                    if select.select([sys.stdin], [], [], 0.01)[0]:
                        next1 = sys.stdin.read(1)
                        if next1 == '[' and select.select([sys.stdin], [], [], 0.01)[0]:
                            next2 = sys.stdin.read(1)
                            
                            # Debounce key presses
                            if current_time - last_key_time > key_debounce:
                                if next2 == 'A':  # UP arrow
                                    selected_index = (selected_index - 1) % len(menu_items)
                                    create_menu_display(selected_index)
                                elif next2 == 'B':  # DOWN arrow
                                    selected_index = (selected_index + 1) % len(menu_items)
                                    create_menu_display(selected_index)
                                last_key_time = current_time
                elif key in 'wWsS' and current_time - last_key_time > key_debounce:
                    if key in 'wW':
                        selected_index = (selected_index - 1) % len(menu_items)
                    else:
                        selected_index = (selected_index + 1) % len(menu_items)
                    create_menu_display(selected_index)
                    last_key_time = current_time
            
            # Small sleep to reduce CPU usage
            time.sleep(0.001)
    
    finally:
        # Restore terminal settings
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        print("\nExiting...")

except ImportError:
    # Fallback without termios
    while True:
        try:
            user_input = input("Press w/s/q then Enter: ").strip().lower()
            
            if user_input == 'q':
                break
            elif user_input == 'w':
                selected_index = (selected_index - 1) % len(menu_items)
                create_menu_display(selected_index)
            elif user_input == 's':
                selected_index = (selected_index + 1) % len(menu_items)
                create_menu_display(selected_index)
        
        except KeyboardInterrupt:
            break

except KeyboardInterrupt:
    pass