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

# Cached resources
_arrow_image = None
_font = None
_base_image = None  # Image with all text (no arrows)
_text_positions = []  # Pre-calculated positions
_menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
_current_selection = 0
_last_selection = -1

def init_resources():
    """Initialize and cache all resources once"""
    global _arrow_image, _font, _base_image, _text_positions
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Load arrow
    if _arrow_image is None:
        arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
        if os.path.exists(arrow_path):
            _arrow_image = Image.open(arrow_path)
            if _arrow_image.mode != "1":
                _arrow_image = _arrow_image.convert("1", dither=Image.NONE)
        else:
            # Create simple arrow
            _arrow_image = Image.new("1", (20, 20), 255)
            draw = ImageDraw.Draw(_arrow_image)
            draw.polygon([(15, 0), (15, 20), (0, 10)], fill=0)
    
    # Load font
    if _font is None:
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            _font = ImageFont.truetype(font_path, 38)
        else:
            _font = ImageFont.load_default()
    
    # Create base image with all text (no arrows) - ONLY ONCE
    if _base_image is None:
        _base_image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(_base_image)
        
        # Calculate positions
        item_height = 45
        total_height = len(_menu_items) * item_height
        start_y = (display.height - total_height) // 2
        
        # Store positions and draw text
        _text_positions = []
        for i, item in enumerate(_menu_items):
            y = start_y + (i * item_height)
            
            # Get text size
            bbox = draw.textbbox((0, 0), item, font=_font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]
            
            # Center text
            x = (display.width - text_width) // 2
            draw.text((x, y), item, font=_font, fill=0)
            
            # Store position for arrow
            _text_positions.append({
                'arrow_x': x - _arrow_image.width - 15,
                'arrow_y': y + (text_height // 2 - _arrow_image.height // 2) + 9
            })

def update_display(selection):
    """Fast display update - only moves the arrow"""
    global _last_selection
    
    # Start with clean base image
    image = _base_image.copy()
    
    # Draw arrow at selected position
    if selection < len(_text_positions):
        pos = _text_positions[selection]
        image.paste(_arrow_image, (pos['arrow_x'], pos['arrow_y']))
    
    # Update display
    display.image(image)
    display.show()
    
    _last_selection = selection

def get_key():
    """Simple blocking key input - most reliable"""
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
    """Main menu handling - optimized for speed"""
    global _current_selection
    
    # Initialize once
    init_resources()
    
    # Initial display
    update_display(_current_selection)
    print(f"Selected: {_menu_items[_current_selection]}")
    
    while True:
        try:
            key = get_key()
            
            if key == 'up':
                _current_selection = (_current_selection - 1) % len(_menu_items)
                update_display(_current_selection)
                print(f"↑ {_menu_items[_current_selection]}")
                
            elif key == 'down':
                _current_selection = (_current_selection + 1) % len(_menu_items)
                update_display(_current_selection)
                print(f"↓ {_menu_items[_current_selection]}")
                
            elif key == '\r' or key == '\n':  # Enter
                print(f"\n✓ SELECTED: {_menu_items[_current_selection]}")
                print("-" * 30)
                
                # Handle selection
                if _current_selection == 0:
                    print("NEW FILE selected")
                elif _current_selection == 1:
                    print("OPEN FILE selected")
                elif _current_selection == 2:
                    print("SETTINGS selected")
                elif _current_selection == 3:
                    print("CREDITS selected")
                
                print("Press any key to continue...")
                get_key()  # Wait for key press
                update_display(_current_selection)  # Redraw
                
            elif key == '\x7f' or key == '\x08':  # Backspace
                print("\nReturning to logo...")
                import logo
                logo.display_logo()
                time.sleep(2)
                return  # Exit menu
                
            elif key == 'q' or key == 'Q':
                print("\nQuitting...")
                break
                
        except KeyboardInterrupt:
            print("\nExiting...")
            break
        except Exception as e:
            print(f"\nError: {e}")
            break

if __name__ == "__main__":
    handle_menu()