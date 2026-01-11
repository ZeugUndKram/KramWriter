"""
Simple BMP Image Viewer for Sharp Memory Display
Use left/right arrow keys or P/N to navigate
"""

import board
import busio
import digitalio
import os
from PIL import Image
import adafruit_sharpmemorydisplay

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Image files
image_files = [
    "Credits_0.bmp",
    "Learn_0.bmp", 
    "Settings_0.bmp",
    "Write_0.bmp",
    "Zeugtris_0.bmp"
]

current_index = 0

def display_centered_image(image_path):
    """Display a BMP image centered on screen"""
    try:
        image = Image.open(image_path)
        
        # Convert to 1-bit if needed
        if image.mode != '1':
            image = image.convert('1')
        
        # Calculate centering
        x_offset = (display.width - image.width) // 2
        y_offset = (display.height - image.height) // 2
        
        # Create blank canvas
        canvas = Image.new("1", (display.width, display.height), 255)
        
        # Paste image centered
        canvas.paste(image, (x_offset, y_offset))
        
        # Display
        display.image(canvas)
        display.show()
        
        print(f"Displaying: {os.path.basename(image_path)}")
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

def main():
    assets_path = "/home/kramwriter/KramWriter/assets/"
    
    # Check if assets folder exists
    if not os.path.exists(assets_path):
        print(f"Trying ./assets/ instead...")
        assets_path = "./assets/"
        if not os.path.exists(assets_path):
            print("Error: Could not find assets folder!")
            return
    
    print(f"Image viewer started. Press:")
    print("  N or Right Arrow - Next image")
    print("  P or Left Arrow  - Previous image")
    print("  Q                - Quit")
    print()
    
    # Display first image
    img_path = os.path.join(assets_path, image_files[current_index])
    if not display_centered_image(img_path):
        print("Failed to display initial image!")
        return
    
    try:
        while True:
            # Simple input (works on most systems)
            try:
                user_input = input("Command [N/P/Q]: ").strip().upper()
            except EOFError:
                break
            
            if user_input == 'Q':
                print("Goodbye!")
                break
            elif user_input == 'N' or user_input == '':
                # Next image
                current_index = (current_index + 1) % len(image_files)
                img_path = os.path.join(assets_path, image_files[current_index])
                display_centered_image(img_path)
            elif user_input == 'P':
                # Previous image
                current_index = (current_index - 1) % len(image_files)
                img_path = os.path.join(assets_path, image_files[current_index])
                display_centered_image(img_path)
            else:
                print("Invalid command. Use N (next), P (previous), or Q (quit)")
                
    except KeyboardInterrupt:
        print("\nExiting...")

if __name__ == "__main__":
    main()