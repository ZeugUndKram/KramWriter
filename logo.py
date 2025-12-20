import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os
import time

# Initialize SPI and CS pin
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
cs = digitalio.DigitalInOut(board.CE0)  # Oder board.CE0 für GPIO 8

# Initialize display
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(
    spi, 
    cs, 
    width=400, 
    height=240,
    baudrate=2000000
)

def display_logo():
    try:
        # Clear display
        display.fill(1)  # White background
        display.show()
        
        # Load logo
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")

        if not os.path.exists(logo_path):
            print(f"Logo nicht gefunden: {logo_path}")
            return False

        # Load and convert image
        logo = Image.open(logo_path).convert("1")
        
        # Create display image
        image = Image.new("1", (400, 240), 1)
        
        # Center logo
        x = (400 - logo.size[0]) // 2
        y = (240 - logo.size[1]) // 2
        image.paste(logo, (x, y))
        
        # Display
        display.image(image)
        display.show()
        
        print("Logo angezeigt!")
        return True

    except Exception as e:
        print(f"Fehler: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    display_logo()
    time.sleep(30)