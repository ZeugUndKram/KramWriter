"""
Simple BMP Image Viewer for Sharp Memory Display
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

def display_image(image_path):
    """Display a BMP image centered on screen"""
    try:
        # Open and convert image
        img = Image.open(image_path)
        
        # Convert to 1-bit if needed
        if img.mode != '1':
            img = img.convert('1')
        
        # Calculate position to center image
        x = (display.width - img.width) // 2
        y = (display.height - img.height) // 2
        
        # Create blank white image
        canvas = Image.new("1", (display.width, display.height), 1)
        
        # Paste the image
        canvas.paste(img, (x, y))
        
        # Display
        display.image(canvas)
        display.show()
        
        print(f"Showing: {os.path.basename(image_path)}")
        return True
        
    except Exception as e:
        print(f"Error with {image_path}: {e}")
        return False

def main():
    # Find assets folder
    assets_folder = None
    for path in ["/assets/", "./assets/", "assets/"]:
        if os.path.exists(path):
            assets_folder = path
            break
    
    if not assets_folder:
        print("Error: No assets folder found!")
        return
    
    print(f"Using assets from: {assets_folder}")
    
    # Check which images exist
    existing_images = []
    for f in image_files:
        full_path = os.path.join(assets_folder, f)
        if os.path.exists(full_path):
            existing_images.append(f)
        else:
            print(f"Missing: {f}")
    
    if not existing_images:
        print("No images found!")
        return
    
    # Show first image
    img_path = os.path.join(assets_folder, existing_images[current_index])
    display_image(img_path)
    
    print("\nControls:")
    print("  N or Enter - Next image")
    print("  P - Previous image")
    print("  Q - Quit")
    print()
    
    while True:
        try:
            cmd = input("Command [N/P/Q]: ").strip().lower()
            
            if cmd == 'q':
                print("Goodbye!")
                break
            elif cmd == 'n' or cmd == '':
                # Next image
                current_index = (current_index + 1) % len(existing_images)
            elif cmd == 'p':
                # Previous image
                current_index = (current_index - 1) % len(existing_images)
            else:
                print("Unknown command. Use N, P, or Q")
                continue
            
            # Display the selected image
            img_path = os.path.join(assets_folder, existing_images[current_index])
            display_image(img_path)
            
        except KeyboardInterrupt:
            print("\nExiting...")
            break
        except Exception as e:
            print(f"Error: {e}")
            break
    
    # Clear display
    display.fill(1)
    display.show()

if __name__ == "__main__":
    main()