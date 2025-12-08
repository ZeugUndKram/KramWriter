import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os

# Initialize display with CS on GPIO 8 (pin 24)
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
cs = digitalio.DigitalInOut(board.D24)  # GPIO 8 for CS (pin 24)
scs = digitalio.DigitalInOut(board.D6)   # Keep SCS on GPIO 6 (pin 22)

# Create display instance - parameters: spi, cs, width, height
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, cs, 400, 240)

def display_logo():
    try:
        # Get the directory where this script is located
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")

        # Check if logo file exists
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            return False

        # Load and convert the BMP image
        logo = Image.open(logo_path)

        # Convert to 1-bit monochrome (required by Sharp display)
        if logo.mode != "1":
            logo = logo.convert("1", dither=Image.NONE)

        # Create display image with white background (1 = white, 0 = black)
        # For Sharp Memory Display: 1=white, 0=black
        image = Image.new("1", (display.width, display.height), 255)

        # Calculate position to center the logo
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2

        # Paste logo onto display image
        image.paste(logo, (x, y))

        # Update display
        display.image(image)
        display.show()

        print("Logo displayed successfully! Press Enter to continue to menu...")
        return True

    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False


if __name__ == "__main__":
    display_logo()

    # Wait for Enter key press
    input("Press Enter to continue to menu...")

    # Switch to menu.py
    script_dir = os.path.dirname(os.path.abspath(__file__))
    menu_path = os.path.join(script_dir, "menu.py")

    if os.path.exists(menu_path):
        print(f"Launching {menu_path}...")
        exec(open(menu_path).read())
    else:
        print(f"menu.py not found at {menu_path}")