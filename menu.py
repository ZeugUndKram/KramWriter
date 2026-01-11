"""
Practical image viewer accepting hardware limits
"""

import board
import busio
import digitalio
import os
import time
from PIL import Image
import adafruit_sharpmemorydisplay

# Quick setup
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Simple cache
cache = []
for f in ["Credits_0.bmp", "Learn_0.bmp", "Settings_0.bmp", "Write_0.bmp", "Zeugtris_0.bmp"]:
    path = os.path.join("assets", f)
    if os.path.exists(path):
        img = Image.open(path).convert('1')
        x = (400 - img.width) // 2
        y = (240 - img.height) // 2
        canvas = Image.new("1", (400, 240), 1)
        canvas.paste(img, (x, y))
        cache.append((f, canvas))
        print(f"✓ {f}")

# Navigation
idx = 0
if cache:
    print(f"\nLoaded {len(cache)} images")
    print("N=Next, P=Prev, Q=Quit")
    print("Note: ~700ms delay is NORMAL for this display\n")
    
    while True:
        name, img = cache[idx]
        print(f"🖼️  Showing: {name} ({idx+1}/{len(cache)})")
        
        start = time.time()
        display.image(img)
        display.show()
        
        cmd = input("Command [N/P/Q]: ").lower()
        
        if cmd == 'q':
            break
        elif cmd == 'n':
            idx = (idx + 1) % len(cache)
        elif cmd == 'p':
            idx = (idx - 1) % len(cache)
else:
    print("No images found in assets/ folder!")