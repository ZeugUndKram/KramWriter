#!/usr/bin/env python3
"""
Spacebar pressed = Show "Hello World" image (like the example)
Spacebar released = White screen
For 400x240 Sharp Memory Display
"""

import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import time
import keyboard  # pip install keyboard

# Initialize display for 400x240
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Colors
BLACK = 0
WHITE = 255

# Parameters to Change (from the example)
BORDER = 5
FONTSIZE = 24  # Increased font size for larger display

def create_hello_world_image():
    """Create the "Hello World" image for 400x240 display"""
    # Create blank image for drawing.
    # Make sure to create image with mode '1' for 1-bit color.
    image = Image.new("1", (display.width, display.height))
    
    # Get drawing object to draw on image.
    draw = ImageDraw.Draw(image)
    
    # Draw a black background (from the example)
    draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=BLACK)
    
    # Draw a smaller inner rectangle (from the example)
    draw.rectangle(
        (BORDER, BORDER, display.width - BORDER - 1, display.height - BORDER - 1),
        outline=WHITE,
        fill=WHITE,
    )
    
    # Load a TTF font with larger size for 400x240 display
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", FONTSIZE)
    except:
        try:
            # Try a different common font path
            font = ImageFont.truetype("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", FONTSIZE)
        except:
            # Fallback to default font if others are not available
            font = ImageFont.load_default()
            print("Note: Using default font")
    
    # Draw Some Text (from the example)
    text = "Hello World!"
    bbox = font.getbbox(text)
    (font_width, font_height) = bbox[2] - bbox[0], bbox[3] - bbox[1]
    draw.text(
        (display.width // 2 - font_width // 2, display.height // 2 - font_height // 2),
        text,
        font=font,
        fill=BLACK,
    )
    
    return image

def main():
    """Main loop"""
    print(f"Display size: {display.width}x{display.height}")
    print("Spacebar pressed = Show 'Hello World' image")
    print("Spacebar released = White screen")
    print("Press Ctrl-C to exit.")
    
    # Create the hello world image
    print("Creating Hello World image...")
    hello_image = create_hello_world_image()
    
    # Start with white screen (using display.fill like in the example)
    display.fill(1)  # Note: 1 for white in display.fill()
    display.show()
    print("Screen: WHITE")
    
    # Track current state: 0 = showing white, 1 = showing hello image
    current_state = 0  # 0 = white, 1 = hello
    
    try:
        while True:
            # Check if space is pressed
            space_pressed = keyboard.is_pressed('space')
            
            # Determine which screen to show
            if space_pressed:
                if current_state != 1:  # Switch to hello image
                    display.image(hello_image)
                    display.show()
                    current_state = 1
                    print("Screen: HELLO WORLD")
            else:
                if current_state != 0:  # Switch to white screen
                    display.fill(1)
                    display.show()
                    current_state = 0
                    print("Screen: WHITE")
            
            # Small delay
            time.sleep(0.01)  # 10ms
            
    except KeyboardInterrupt:
        pass
    finally:
        # Clear display to white on exit
        display.fill(1)
        display.show()
        print("\nExiting...")

if __name__ == "__main__":
    main()