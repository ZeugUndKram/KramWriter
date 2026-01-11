#!/usr/bin/env python3
"""
Event-driven display controller using proper image handling
Spacebar pressed = Black screen, Released = White screen
"""

import board
import busio
import digitalio
from PIL import Image, ImageDraw
import adafruit_sharpmemorydisplay
from pynput import keyboard

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)

# Colors
BLACK = 0
WHITE = 255

def create_blank_image(fill_color):
    """Create a blank image with the specified color"""
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), 
                   outline=fill_color, fill=fill_color)
    return image

class DisplayController:
    def __init__(self):
        # Create images upfront
        self.black_image = create_blank_image(BLACK)
        self.white_image = create_blank_image(WHITE)
        
        # Start with white screen
        self.current_image = self.white_image
        display.image(self.white_image)
        display.show()
        print("Screen: WHITE")
    
    def update_display(self, is_black):
        """Update the display with the appropriate image"""
        if is_black:
            if self.current_image != self.black_image:
                display.image(self.black_image)
                display.show()
                self.current_image = self.black_image
                print("Screen: BLACK")
        else:
            if self.current_image != self.white_image:
                display.image(self.white_image)
                display.show()
                self.current_image = self.white_image
                print("Screen: WHITE")
    
    def on_press(self, key):
        """Handle key press"""
        try:
            if key == keyboard.Key.space:
                self.update_display(True)
        except AttributeError:
            pass
    
    def on_release(self, key):
        """Handle key release"""
        try:
            if key == keyboard.Key.space:
                self.update_display(False)
            elif key == keyboard.Key.esc:
                return False  # Stop listener
        except AttributeError:
            pass

def main():
    """Main function"""
    controller = DisplayController()
    
    print("Press space to make screen black, release for white.")
    print("Press ESC to exit.")
    
    # Start keyboard listener
    with keyboard.Listener(
        on_press=controller.on_press,
        on_release=controller.on_release) as listener:
        
        listener.join()
    
    # Clear to white on exit
    display.image(controller.white_image)
    display.show()
    print("\nExiting...")

if __name__ == "__main__":
    main()