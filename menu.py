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
        
        # Try to use a font, fall back to default if not available
        try:
            font = ImageFont.load_default()
        except:
            font = ImageFont.load_default()
        
        # Menu items
        menu_items = [
            "new file",
            "open file", 
            "settings",
            "credits"
        ]
        
        # Calculate positions
        total_height = len(menu_items) * 30  # 30 pixels per item
        start_y = (display.height - total_height) // 2
        
        # Draw each menu item
        for i, item in enumerate(menu_items):
            y_position = start_y + (i * 30)
            
            # Draw the menu text
            draw.text((50, y_position), item, font=font, fill=0)
            
            # Draw a simple bullet point or number
            draw.text((30, y_position), f"{i+1}.", font=font, fill=0)
        
        # Update display
        display.image(image)
        display.show()
        
        print("Menu displayed successfully!")
        print("1. new file")
        print("2. open file") 
        print("3. settings")
        print("4. credits")
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
                print("Selected: new file")
                # Add your new file functionality here
                break
            elif selection == '2':
                print("Selected: open file")
                # Add your open file functionality here
                break
            elif selection == '3':
                print("Selected: settings")
                # Add your settings functionality here
                break
            elif selection == '4':
                print("Selected: credits")
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