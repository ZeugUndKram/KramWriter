import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os

# Initialize display once
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def display_logo_optimized():
    try:
        logo_path = os.path.join(os.path.dirname(__file__), "assets", "logo.bmp")
        
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            return False
        
        # Pre-convert your logo to 1-bit BMP to avoid runtime conversion
        logo = Image.open(logo_path)
        
        # Create image and paste in one operation
        image = Image.new("1", (display.width, display.height), 255)
        x = (display.width - logo.width) // 2
        y = (display.height - logo.height) // 2
        image.paste(logo, (x, y))
        
        # Single display update
        display.image(image)
        display.show()
        
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    display_logo_optimized()