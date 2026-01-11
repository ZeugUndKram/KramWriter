"""
Smart viewer that shows animation during loading, then static image
"""

import board
import busio
import digitalio
import os
import time
from PIL import Image, ImageDraw
import adafruit_sharpmemorydisplay

# Initialize
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

class SmartViewer:
    def __init__(self):
        self.images = []
        self.load_all()
    
    def load_all(self):
        """Load all images with progress indicator"""
        files = ["Credits_0.bmp", "Learn_0.bmp", "Settings_0.bmp", "Write_0.bmp", "Zeugtris_0.bmp"]
        
        for i, f in enumerate(files):
            path = os.path.join("assets", f)
            if os.path.exists(path):
                # Show loading animation
                self.show_loading(i, len(files), f)
                
                # Load image
                img = Image.open(path).convert('1')
                x = (400 - img.width) // 2
                y = (240 - img.height) // 2
                canvas = Image.new("1", (400, 240), 1)
                canvas.paste(img, (x, y))
                self.images.append((f, canvas))
        
        print(f"Loaded {len(self.images)} images")
    
    def show_loading(self, current, total, name):
        """Show animated loading screen"""
        canvas = Image.new("1", (400, 240), 1)
        draw = ImageDraw.Draw(canvas)
        
        # Progress bar
        bar_width = 300
        bar_height = 20
        bar_x = 50
        bar_y = 150
        
        # Background
        draw.rectangle([bar_x, bar_y, bar_x+bar_width, bar_y+bar_height], outline=0)
        
        # Progress
        progress = (current + 1) / total
        fill_width = int(bar_width * progress)
        draw.rectangle([bar_x, bar_y, bar_x+fill_width, bar_y+bar_height], fill=0)
        
        # Text
        draw.text((200, 100), f"Loading...", fill=0, anchor="mm")
        draw.text((200, 180), f"{name}", fill=0, anchor="mm")
        
        display.image(canvas)
        display.show()
        time.sleep(0.1)  # Brief pause to show progress
    
    def display(self, index):
        """Display pre-loaded image instantly"""
        if 0 <= index < len(self.images):
            name, img = self.images[index]
            
            # Show immediately
            start = time.time()
            display.image(img)
            display.show()
            elapsed = time.time() - start
            
            print(f"Displayed {name} in {elapsed*1000:.0f}ms")
            return True
        return False