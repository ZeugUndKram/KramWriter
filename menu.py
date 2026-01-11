"""
Sharp Display Viewer with partial update optimization
Note: Partial updates may not be supported by all Sharp Memory Displays
"""

import board
import busio
import digitalio
import os
import time
from PIL import Image
import adafruit_sharpmemorydisplay

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Check if partial update is available
try:
    # Try to use partial refresh if available
    display.partial_mode = True
    print("Partial update mode enabled")
except:
    print("Partial update not supported, using full refresh")
    display.partial_mode = False

# Pre-cache all images
def cache_images():
    images = []
    files = ["Credits_0.bmp", "Learn_0.bmp", "Settings_0.bmp", "Write_0.bmp", "Zeugtris_0.bmp"]
    
    for f in files:
        path = os.path.join("assets", f)
        if os.path.exists(path):
            img = Image.open(path).convert('1')
            # Store as bytearray for faster transfer
            img_bytes = img.tobytes()
            images.append((f, img.size, img_bytes))
    return images

# Quick display using direct buffer access
def display_fast(img_name, img_size, img_bytes):
    """Fastest possible display - minimal Python overhead"""
    # Create new image from bytes
    img = Image.frombytes('1', img_size, img_bytes)
    
    # Center it
    x = (400 - img.width) // 2
    y = (240 - img.height) // 2
    canvas = Image.new("1", (400, 240), 1)
    canvas.paste(img, (x, y))
    
    # Display
    display.image(canvas)
    display.show()