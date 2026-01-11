"""
Ultra-fast BMP viewer - minimal overhead
"""

import board
import busio
import digitalio
import os
from PIL import Image
import adafruit_sharpmemorydisplay

# Initialize with faster SPI if possible
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
# Try to increase SPI speed (if supported)
try:
    spi.try_lock()
    spi.configure(baudrate=8000000)  # 8 MHz
    spi.unlock()
except:
    pass

scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Image cache
image_cache = []

def cache_images(folder="assets"):
    """Load and cache all images once"""
    files = ["Credits_0.bmp", "Learn_0.bmp", "Settings_0.bmp", "Write_0.bmp", "Zeugtris_0.bmp"]
    
    for f in files:
        path = os.path.join(folder, f)
        if os.path.exists(path):
            img = Image.open(path)
            if img.mode != '1':
                img = img.convert('1')
            
            # Center and cache
            x = (400 - img.width) // 2
            y = (240 - img.height) // 2
            canvas = Image.new("1", (400, 240), 1)
            canvas.paste(img, (x, y))
            image_cache.append(canvas)
            print(f"Cached: {f}")
        else:
            print(f"Missing: {f}")
    
    return len(image_cache)

# Main loop
if cache_images() > 0:
    idx = 0
    display.image(image_cache[idx])
    display.show()
    
    while True:
        cmd = input(f"Image {idx+1}/{len(image_cache)} [N/P/Q]: ").lower()
        
        if cmd == 'q':
            break
        elif cmd == 'n' or cmd == '':
            idx = (idx + 1) % len(image_cache)
        elif cmd == 'p':
            idx = (idx - 1) % len(image_cache)
        
        display.image(image_cache[idx])
        display.show()