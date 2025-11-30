#!/usr/bin/env python3
import os
import sys
import time


def initialize_display():
    """Safely initialize display with error handling"""
    try:
        import board
        import busio
        import digitalio
        from PIL import Image
        import adafruit_sharpmemorydisplay

        spi = busio.SPI(board.SCK, MOSI=board.MOSI)
        scs = digitalio.DigitalInOut(board.D6)
        display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)
        return display, None
    except Exception as e:
        return None, f"Display initialization failed: {e}"


def display_logo():
    display, error = initialize_display()
    if error:
        print(f"ERROR: {error}")
        return False

    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")

        if not os.path.exists(logo_path):
            print(f"ERROR: Logo file not found: {logo_path}")
            print("Please ensure assets/logo.bmp exists")
            return False

        logo = Image.open(logo_path)
        print(f"Loaded logo: {logo.size[0]}x{logo.size[1]}, mode: {logo.mode}")

        if logo.mode != "1":
            logo = logo.convert("1", dither=Image.NONE)

        image = Image.new("1", (display.width, display.height), 255)
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        image.paste(logo, (x, y))

        display.image(image)
        display.show()

        print("SUCCESS: Logo displayed")
        return True

    except Exception as e:
        print(f"ERROR displaying logo: {e}")
        return False


def main():
    print("=== Logo Display ===")
    success = display_logo()

    if success:
        print("Logo displayed successfully!")
        input("Press Enter to return to launcher...")
    else:
        print("Failed to display logo")
        input("Press Enter to continue...")


if __name__ == "__main__":
    main()