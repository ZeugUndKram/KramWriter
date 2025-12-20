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

# State
arrow_image = None
font = None
menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
current_selection = 0
last_selection = -1
text_positions = []

def setup():
    """One-time setup of all resources"""
    global arrow_image, font, text_positions
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Load arrow
    arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
    if os.path.exists(arrow_path):
        arrow_image = Image.open(arrow_path)
        if arrow_image.mode != "1":
            arrow_image = arrow_image.convert("1", dither=Image.NONE)
    else:
        arrow_image = Image.new("1", (20, 20), 255)
        draw = ImageDraw.Draw(arrow_image)
        draw.polygon([(15, 0), (15, 20), (0, 10)], fill=0)
    
    # Load font
    font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
    if os.path.exists(font_path):
        font = ImageFont.truetype(font_path, 38)
    else:
        font = ImageFont.load_default()
    
    # Pre-calculate all text positions (ONE TIME)
    temp_image = Image.new("1", (display.width, display.height), 255)
    temp_draw = ImageDraw.Draw(temp_image)
    
    item_height = 45
    total_height = len(menu_items) * item_height
    start_y = (display.height - total_height) // 2
    
    text_positions = []
    for i, item in enumerate(menu_items):
        y = start_y + (i * item_height)
        bbox = temp_draw.textbbox((0, 0), item, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        x = (display.width - text_width) // 2
        
        text_positions.append({
            'x': x,
            'y': y,
            'text_width': text_width,
            'text_height': text_height,
            'arrow_x': x - arrow_image.width - 15,
            'arrow_y': y + (text_height // 2 - arrow_image.height // 2) + 9
        })

def draw_initial_menu():
    """Draw complete menu once"""
    # Create clean white background
    image = Image.new("1", (display.width, display.height), 255)
    draw = ImageDraw.Draw(image)
    
    # Draw all menu items
    for i, item in enumerate(menu_items):
        pos = text_positions[i]
        draw.text((pos['x'], pos['y']), item, font=font, fill=0)
    
    # Draw initial arrow
    pos = text_positions[current_selection]
    image.paste(arrow_image, (pos['arrow_x'], pos['arrow_y']))
    
    # Show on display
    display.image(image)
    display.show()

def update_arrow_position(new_selection):
    """Fast update: only move the arrow"""
    global current_selection, last_selection
    
    if new_selection == last_selection:
        return
    
    # Clear old arrow (draw white rectangle over it)
    if last_selection >= 0:
        old_pos = text_positions[last_selection]
        display.fill_rect(
            old_pos['arrow_x'] - 2, old_pos['arrow_y'] - 2,
            arrow_image.width + 4, arrow_image.height + 4,
            1  # White
        )
    
    # Draw new arrow
    new_pos = text_positions[new_selection]
    display.bitmap(
        new_pos['arrow_x'], new_pos['arrow_y'],
        arrow_image.width, arrow_image.height,
        arrow_image.getdata(), 1
    )
    
    # Show updates
    display.show()
    
    # Update state
    last_selection = current_selection
    current_selection = new_selection

def get_key():
    """Simple, reliable blocking input"""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
        
        if ch == '\x1b':  # Escape sequence for arrows
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

def run_menu():
    """Main menu loop - simple and fast"""
    global current_selection
    
    # Setup once
    setup()
    
    # Draw initial menu
    draw_initial_menu()
    last_selection = current_selection
    
    print("MENU: ↑↓ = Navigate, Enter = Select, Backspace = Logo, Q = Quit")
    print(f"Current: {menu_items[current_selection]}")
    
    while True:
        try:
            key = get_key()
            
            if key == 'up':
                new_sel = (current_selection - 1) % len(menu_items)
                update_arrow_position(new_sel)
                print(f"↑ {menu_items[new_sel]}")
                
            elif key == 'down':
                new_sel = (current_selection + 1) % len(menu_items)
                update_arrow_position(new_sel)
                print(f"↓ {menu_items[new_sel]}")
                
            elif key == '\r' or key == '\n':  # Enter
                print(f"\n✓ SELECTED: {menu_items[current_selection]}")
                
                if current_selection == 0:
                    print("NEW FILE functionality")
                elif current_selection == 1:
                    print("OPEN FILE functionality")
                elif current_selection == 2:
                    print("SETTINGS functionality")
                elif current_selection == 3:
                    print("CREDITS functionality")
                
                # Brief pause then continue
                time.sleep(0.3)
                print("Menu ready...")
                
            elif key == '\x7f' or key == '\x08':  # Backspace
                print("\nReturning to logo...")
                import logo
                logo.display_logo()
                time.sleep(2)
                return  # Exit menu
                
            elif key == 'q' or key == 'Q':
                print("\nQuitting menu...")
                break
                
            else:
                print(f"Key: {repr(key)}")
                
        except KeyboardInterrupt:
            print("\nMenu cancelled")
            break
        except Exception as e:
            print(f"Error: {e}")
            break

if __name__ == "__main__":
    run_menu()