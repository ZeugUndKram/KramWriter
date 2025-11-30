import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def display_logo():
    try:
        # Get the directory where this script is located
        script_dir = os.path.dirname(os.path.abspath(__file__))
        graphics_dir = os.path.join(script_dir, "assets")
        logo_path = os.path.join(graphics_dir, "logo.bmp")
        
        # Check if logo file exists
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            print("Please create a 'graphics' folder with 'logo.bmp'")
            return False
        
        # Load the BMP image
        logo = Image.open(logo_path)
        print(f"Loaded logo: {logo.size[0]}x{logo.size[1]}, mode: {logo.mode}")
        
        # Create display image with white background
        image = Image.new("1", (display.width, display.height), 255)
        
        # Calculate position to center the logo
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        
        # Paste logo onto display image
        image.paste(logo, (x, y))
        
        # Update display
        display.image(image)
        display.show()
        
        print("Logo displayed successfully!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False

if __name__ == "__main__":
    print("=== BMP Logo Display ===")
    success = display_logo()
    
    if success:
        print("Logo is now on display!")
    else:
        print("Failed to display logo")