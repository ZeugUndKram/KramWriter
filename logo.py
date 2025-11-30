import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os
import subprocess
import sys

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)


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

        # Convert to 1-bit monochrome
        if logo.mode != "1":
            logo = logo.convert("1", dither=Image.NONE)

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

        print("Logo displayed successfully! Press Enter to continue to menu...")
        return True

    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False


if __name__ == "__main__":
    display_logo()

    # Wait for Enter key press
    input("Press Enter to continue to menu...")

    # Switch to menu.py using subprocess
    script_dir = os.path.dirname(os.path.abspath(__file__))
    menu_path = os.path.join(script_dir, "menu.py")

    if os.path.exists(menu_path):
        print(f"Launching {menu_path}...")
        subprocess.run([sys.executable, menu_path])
    else:
        print(f"menu.py not found at {menu_path}")