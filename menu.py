import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os
import time
import sys
import termios
import tty

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Global variables
arrow_image = None
font = None
menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
current_selection = 0

def load_resources():
    """Load images and fonts once"""
    global arrow_image, font
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Load arrow
    arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
    if os.path.exists(arrow_path):
        arrow_image = Image.open(arrow_path)
        if arrow_image.mode != "1":
            arrow_image = arrow_image.convert("1", dither=Image.NONE)
    else:
        print(f"Arrow not found: {arrow_path}")
        # Create simple arrow
        arrow_image = Image.new("1", (20, 20), 255)
        draw = ImageDraw.Draw(arrow_image)
        draw.polygon([(15, 0), (15, 20), (0, 10)], fill=0)
    
    # Load font
    font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
    if os.path.exists(font_path):
        font = ImageFont.truetype(font_path, 38)
    else:
        print(f"Font not found: {font_path}")
        font = ImageFont.load_default()

def draw_menu(selection=0):
    """Draw the menu with selection highlight"""
    # Create blank white image
    image = Image.new("1", (display.width, display.height), 255)
    draw = ImageDraw.Draw(image)
    
    # Calculate positions
    item_height = 45
    total_height = len(menu_items) * item_height
    start_y = (display.height - total_height) // 2
    
    # Draw each menu item
    for i, item in enumerate(menu_items):
        y = start_y + (i * item_height)
        
        # Get text size
        bbox = draw.textbbox((0, 0), item, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        
        # Center text
        x = (display.width - text_width) // 2
        draw.text((x, y), item, font=font, fill=0)
        
        # Draw arrow for selected item
        if i == selection and arrow_image:
            arrow_x = x - arrow_image.width - 15
            arrow_y = y + (text_height // 2 - arrow_image.height // 2) + 9
            image.paste(arrow_image, (arrow_x, arrow_y))
    
    # Update display
    display.image(image)
    display.show()

def get_key():
    """Simple blocking key input"""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
        
        # Check for arrow keys
        if ch == '\x1b':
            # Read the next 2 characters
            next_ch = sys.stdin.read(1)
            if next_ch == '[':
                arrow_ch = sys.stdin.read(1)
                if arrow_ch == 'A':
                    return 'up'
                elif arrow_ch == 'B':
                    return 'down'
        
        return ch
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

def handle_menu():
    """Main menu handling function"""
    global current_selection
    
    # Load resources
    load_resources()
    
    print("=== MENU ===")
    print("Arrow Keys: Navigate")
    print("Enter: Select")
    print("Backspace: Return to logo")
    print("Q: Quit")
    print("=" * 11)
    
    # Initial draw
    draw_menu(current_selection)
    print(f"Selected: {menu_items[current_selection]}")
    
    while True:
        try:
            print("\nWaiting for input...")
            key = get_key()
            
            if key == 'up':
                current_selection = (current_selection - 1) % len(menu_items)
                draw_menu(current_selection)
                print(f"↑ {menu_items[current_selection]}")
                
            elif key == 'down':
                current_selection = (current_selection + 1) % len(menu_items)
                draw_menu(current_selection)
                print(f"↓ {menu_items[current_selection]}")
                
            elif key == '\r' or key == '\n':  # Enter
                print(f"\n✓ SELECTED: {menu_items[current_selection]}")
                print("-" * 30)
                
                # Handle selection
                if current_selection == 0:
                    print("NEW FILE selected")
                    # Add your new file code here
                elif current_selection == 1:
                    print("OPEN FILE selected")
                    # Add your open file code here
                elif current_selection == 2:
                    print("SETTINGS selected")
                    # Add your settings code here
                elif current_selection == 3:
                    print("CREDITS selected")
                    # Add your credits code here
                
                print("Press any key to continue...")
                get_key()  # Wait for key press
                draw_menu(current_selection)  # Redraw menu
                
            elif key == '\x7f' or key == '\x08':  # Backspace
                print("\nReturning to logo...")
                import logo
                logo.display_logo()
                time.sleep(2)
                return  # Exit menu, back to logo
                
            elif key == 'q' or key == 'Q':
                print("\nQuitting...")
                break
                
            else:
                print(f"Key pressed: {repr(key)} (not a menu command)")
                
        except KeyboardInterrupt:
            print("\n\nExiting...")
            break
        except Exception as e:
            print(f"\nError: {e}")
            break

if __name__ == "__main__":
    handle_menu()