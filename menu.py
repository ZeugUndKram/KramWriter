import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os
import time

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def display_menu():
    try:
        # Create display image with white background
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)
        
        # Menu items
        menu_items = [
            "NEW FILE",
            "OPEN FILE", 
            "SETTINGS",
            "CREDITS"
        ]
        
        # Try to load larger fonts, fall back to scaling if not available
        try:
            # Try to load a larger font - common paths for built-in fonts
            font_paths = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
                "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
            ]
            font = None
            for path in font_paths:
                if os.path.exists(path):
                    font = ImageFont.truetype(path, 24)  # Larger font size
                    break
        except:
            font = None
        
        # If no larger font found, we'll use the default and make it bigger by scaling
        if font is None:
            try:
                font = ImageFont.load_default()
                print("Using default font (may be small)")
            except:
                font = None
        
        # Calculate text dimensions and positions
        item_height = 50  # Even more height for bigger text
        total_height = len(menu_items) * item_height
        start_y = (display.height - total_height) // 2
        
        # Draw each menu item centered with bigger text
        for i, item in enumerate(menu_items):
            y_position = start_y + (i * item_height)
            
            if font:
                # Get text bounding box to center it
                bbox = draw.textbbox((0, 0), item, font=font)
                text_width = bbox[2] - bbox[0]
                text_height = bbox[3] - bbox[1]
                
                x_position = (display.width - text_width) // 2
                
                # Draw the menu text with larger font
                draw.text((x_position, y_position), item, font=font, fill=0)
            else:
                # Fallback: Draw bigger text by using larger coordinates and thicker lines
                # Estimate text width (approx 15 pixels per character for bigger text)
                text_width = len(item) * 15
                x_position = (display.width - text_width) // 2
                
                # Draw thicker text by drawing multiple times with slight offsets
                for dx, dy in [(0,0), (1,0), (0,1), (1,1)]:
                    draw.text((x_position + dx, y_position + dy), item, fill=0)
        
        # Update display
        display.image(image)
        display.show()
        
        print("Menu displayed successfully!")
        print("1. NEW FILE")
        print("2. OPEN FILE") 
        print("3. SETTINGS")
        print("4. CREDITS")
        print("Press backspace to return to logo")
        
        return True
        
    except Exception as e:
        print(f"Error displaying menu: {e}")
        return False

def handle_menu_selection():
    """Wait for user input and handle menu selection"""
    print("\nSelect an option (1-4), 'q' to quit, or backspace to return to logo:")
    
    while True:
        try:
            selection = input().strip().lower()
            
            if selection == '1':
                print("Selected: NEW FILE")
                # Add your new file functionality here
                break
            elif selection == '2':
                print("Selected: OPEN FILE")
                # Add your open file functionality here
                break
            elif selection == '3':
                print("Selected: SETTINGS")
                # Add your settings functionality here
                break
            elif selection == '4':
                print("Selected: CREDITS")
                # Add your credits functionality here
                break
            elif selection == 'q':
                print("Quitting menu...")
                break
            elif selection == '' or selection == '\x08':  # Backspace or empty input
                print("Returning to logo...")
                # Return to logo.py
                script_dir = os.path.dirname(os.path.abspath(__file__))
                logo_path = os.path.join(script_dir, "logo.py")
                
                if os.path.exists(logo_path):
                    print(f"Returning to {logo_path}...")
                    exec(open(logo_path).read())
                    return  # Exit after launching logo.py
                else:
                    print(f"logo.py not found at {logo_path}")
                break
            else:
                print("Invalid selection. Please choose 1-4, 'q' to quit, or backspace to return to logo:")
                
        except KeyboardInterrupt:
            print("\nMenu interrupted")
            break
        except Exception as e:
            print(f"Error handling selection: {e}")
            break

if __name__ == "__main__":
    print("=== Menu ===")
    success = display_menu()
    
    if success:
        handle_menu_selection()
    else:
        print("Failed to display menu")