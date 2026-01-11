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

# Create blank image for drawing.
# Make sure to create image with mode '1' for 1-bit color.
image = Image.new("1", (display.width, display.height))

# Get drawing object to draw on image.
draw = ImageDraw.Draw(image)

# Draw a black background
draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=WHITE)

# Define text elements with their font sizes
text_elements = [
    ("Zeugtris", 30),      # First element - size 30
    ("Element 2", 40),     # Second element - size 40
    ("Element 3", 50),     # Third element - size 50
    ("Element 4", 40),     # Fourth element - size 40
    ("Final", 30)          # Fifth element - size 30
]

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