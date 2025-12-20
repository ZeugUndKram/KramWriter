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
_display_buffer = None  # Current display buffer
_text_positions = []  # Pre-calculated positions
_menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
_current_selection = 0

def init_resources():
    """Initialize and cache all resources once"""
    global _arrow_image, _font, _base_image, _display_buffer, _text_positions
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Load arrow (small and optimized)
    if _arrow_image is None:
        arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
        if os.path.exists(arrow_path):
            _arrow_image = Image.open(arrow_path)
            if _arrow_image.mode != "1":
                _arrow_image = _arrow_image.convert("1", dither=Image.NONE)
        else:
            # Create minimal arrow
            _arrow_image = Image.new("1", (15, 10), 255)  # Smaller!
            draw = ImageDraw.Draw(_arrow_image)
            draw.polygon([(10, 0), (10, 10), (0, 5)], fill=0)
    
    # Load font
    if _font is None:
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            _font = ImageFont.truetype(font_path, 36)  # Slightly smaller font
        else:
            _font = ImageFont.load_default()
    
    # Create base image with all text
    if _base_image is None:
        _base_image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(_base_image)
        
        # Calculate positions
        item_height = 40  # Reduced from 45
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
            
            # Store position for arrow with padding
            arrow_x = x - _arrow_image.width - 10  # Reduced padding
            arrow_y = y + (text_height // 2 - _arrow_image.height // 2) + 5
            
            # Define arrow area to clear (white rectangle)
            clear_area = (
                arrow_x - 1, arrow_y - 1,
                arrow_x + _arrow_image.width + 1,
                arrow_y + _arrow_image.height + 1
            )
            
            _text_positions.append({
                'arrow_x': arrow_x,
                'arrow_y': arrow_y,
                'clear_area': clear_area
            })
        
        # Create initial display buffer
        _display_buffer = _base_image.copy()
        draw = ImageDraw.Draw(_display_buffer)
        
        # Draw initial arrow
        pos = _text_positions[0]
        _display_buffer.paste(_arrow_image, (pos['arrow_x'], pos['arrow_y']))
        
        # Display initial image
        display.image(_display_buffer)
        display.show()

def move_arrow(new_selection):
    """ULTRA FAST: Only update arrow position by clearing old and drawing new"""
    global _current_selection, _display_buffer
    
    if new_selection == _current_selection:
        return
    
    # Get positions
    old_pos = _text_positions[_current_selection]
    new_pos = _text_positions[new_selection]
    
    # Clear old arrow (draw white rectangle)
    display_rect = old_pos['clear_area']
    display.fill_rect(
        display_rect[0], display_rect[1],
        display_rect[2] - display_rect[0] + 1,
        display_rect[3] - display_rect[1] + 1,
        1  # White
    )
    
    # Draw new arrow
    display.bitmap(
        new_pos['arrow_x'], new_pos['arrow_y'],
        _arrow_image.width, _arrow_image.height,
        _arrow_image.getdata(), 1
    )
    
    # Update buffer in memory
    draw = ImageDraw.Draw(_display_buffer)
    draw.rectangle(old_pos['clear_area'], fill=255)
    _display_buffer.paste(_arrow_image, (new_pos['arrow_x'], new_pos['arrow_y']))
    
    # Show updates
    display.show()
    _current_selection = new_selection

def get_key():
    """Non-blocking key check"""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    
    try:
        tty.setraw(fd)
        # Set non-blocking
        import fcntl
        fl = fcntl.fcntl(fd, fcntl.F_GETFL)
        fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
        
        try:
            ch = sys.stdin.read(1)
            
            # Check for arrow keys
            if ch == '\x1b':
                try:
                    next_ch = sys.stdin.read(1)
                    if next_ch == '[':
                        arrow_ch = sys.stdin.read(1)
                        if arrow_ch == 'A':
                            return 'up'
                        elif arrow_ch == 'B':
                            return 'down'
                except:
                    pass
            
            return ch if ch else None
        except:
            return None
    finally:
        fcntl.fcntl(fd, fcntl.F_SETFL, fl)
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

def handle_menu():
    """Main menu - optimized for maximum speed"""
    global _current_selection
    
    # Initialize once
    init_resources()
    
    # Skip initial print to save time
    # print(f"Selected: {_menu_items[_current_selection]}")
    
    # Buffer for debouncing
    last_update = time.time()
    update_delay = 0.05  # 50ms minimum between updates
    
    while True:
        try:
            key = get_key()
            
            if not key:
                time.sleep(0.01)  # Small sleep to reduce CPU
                continue
            
            current_time = time.time()
            
            if key == 'up':
                new_selection = (_current_selection - 1) % len(_menu_items)
                if current_time - last_update >= update_delay:
                    move_arrow(new_selection)
                    # print(f"↑ {_menu_items[new_selection]}")
                    last_update = current_time
                
            elif key == 'down':
                new_selection = (_current_selection + 1) % len(_menu_items)
                if current_time - last_update >= update_delay:
                    move_arrow(new_selection)
                    # print(f"↓ {_menu_items[new_selection]}")
                    last_update = current_time
                
            elif key == '\r' or key == '\n':  # Enter
                print(f"\n✓ {_menu_items[_current_selection]}")
                
                # Handle selection
                if _current_selection == 0:
                    print("NEW FILE")
                elif _current_selection == 1:
                    print("OPEN FILE")
                elif _current_selection == 2:
                    print("SETTINGS")
                elif _current_selection == 3:
                    print("CREDITS")
                
                # Wait a moment and continue
                time.sleep(0.5)
                
            elif key == '\x7f' or key == '\x08':  # Backspace
                print("\nBack to logo...")
                import logo
                logo.display_logo()
                time.sleep(1)
                return
                
            elif key == 'q' or key == 'Q':
                print("\nQuit")
                break
                
        except KeyboardInterrupt:
            break
        except Exception as e:
            print(f"Error: {e}")
            break

if __name__ == "__main__":
    handle_menu()