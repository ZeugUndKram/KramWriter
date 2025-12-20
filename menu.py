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
import select

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# CACHED GLOBALS
_arrow_image = None
_font = None
_menu_cache = {}  # Cache rendered menu images by selected_index
_menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
_item_height = 45
_last_selected_index = -1  # Track last selected index to clear previous arrow

def _init_resources():
    """Initialize and cache expensive resources once"""
    global _arrow_image, _font
    
    if _arrow_image is None:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
        
        if os.path.exists(arrow_path):
            _arrow_image = Image.open(arrow_path)
            if _arrow_image.mode != "1":
                _arrow_image = _arrow_image.convert("1", dither=Image.NONE)
        else:
            print(f"Arrow image not found: {arrow_path}")
            # Create a simple arrow as fallback
            _arrow_image = Image.new("1", (20, 20), 255)
            draw = ImageDraw.Draw(_arrow_image)
            draw.polygon([(15, 0), (15, 20), (0, 10)], fill=0)
    
    if _font is None:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        
        if os.path.exists(font_path):
            _font = ImageFont.truetype(font_path, 38)
        else:
            print(f"Custom font not found: {font_path}")
            _font = ImageFont.load_default()
    
    # Pre-calculate text dimensions for all menu items
    if not hasattr(_font, '_text_dimensions'):
        _font._text_dimensions = []
        temp_draw = ImageDraw.Draw(Image.new("1", (1, 1)))
        for item in _menu_items:
            bbox = temp_draw.textbbox((0, 0), item, font=_font)
            _font._text_dimensions.append({
                'width': bbox[2] - bbox[0],
                'height': bbox[3] - bbox[1],
                'bbox': bbox  # Store full bbox for calculations
            })

def _render_menu_image(selected_index=0):
    """Render menu image with arrow at selected position"""
    global _last_selected_index
    
    # Check cache first (only if we don't need to clear previous arrow)
    if selected_index in _menu_cache and _last_selected_index == -1:
        return _menu_cache[selected_index]
    
    # Initialize resources if needed
    _init_resources()
    
    # Create base menu image or get cached version
    if "base_image" not in _menu_cache:
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)
        
        # Calculate total height and starting Y
        total_height = len(_menu_items) * _item_height
        start_y = (display.height - total_height) // 2
        
        # Draw all menu items (without arrows)
        for i, item in enumerate(_menu_items):
            y_position = start_y + (i * _item_height)
            dim = _font._text_dimensions[i]
            x_position = (display.width - dim['width']) // 2
            draw.text((x_position, y_position), item, font=_font, fill=0)
        
        _menu_cache["base_image"] = image
        _menu_cache["start_y"] = start_y
        _menu_cache["image_copy"] = image.copy()  # Keep a clean copy
    else:
        # Start with clean base image (no arrows)
        image = _menu_cache["image_copy"].copy()
        draw = ImageDraw.Draw(image)
        start_y = _menu_cache["start_y"]
    
    # Clear previous arrow if exists
    if _last_selected_index >= 0 and _last_selected_index < len(_menu_items):
        prev_dim = _font._text_dimensions[_last_selected_index]
        prev_y = start_y + (_last_selected_index * _item_height)
        prev_x = (display.width - prev_dim['width']) // 2
        prev_arrow_x = prev_x - _arrow_image.width - 15
        prev_arrow_y = prev_y + (prev_dim['height'] // 2 - _arrow_image.height // 2) + 9
        
        # Clear previous arrow area (draw white rectangle)
        draw.rectangle(
            (prev_arrow_x - 2, prev_arrow_y - 2, 
             prev_arrow_x + _arrow_image.width + 2, 
             prev_arrow_y + _arrow_image.height + 2),
            fill=255, outline=255
        )
    
    # Draw new arrow for current selection
    if _arrow_image:
        dim = _font._text_dimensions[selected_index]
        y_position = start_y + (selected_index * _item_height)
        x_position = (display.width - dim['width']) // 2
        
        arrow_x = x_position - _arrow_image.width - 15
        arrow_y = y_position + (dim['height'] // 2 - _arrow_image.height // 2) + 9
        
        # Paste new arrow
        image.paste(_arrow_image, (arrow_x, arrow_y))
    
    # Update last selected index
    _last_selected_index = selected_index
    
    # Cache the result
    _menu_cache[selected_index] = image
    return image

def display_menu(selected_index=0):
    """Optimized display update - moves arrow between options"""
    try:
        # Get rendered image with arrow at correct position
        image = _render_menu_image(selected_index)
        
        # Update display
        display.image(image)
        display.show()
        
        return True
        
    except Exception as e:
        print(f"Error displaying menu: {e}")
        import traceback
        traceback.print_exc()
        return False

class InputHandler:
    """Handles non-blocking keyboard input"""
    
    def __init__(self):
        self.fd = sys.stdin.fileno()
        self.old_settings = termios.tcgetattr(self.fd)
        self._setup_raw_mode()
        
    def _setup_raw_mode(self):
        """Set terminal to raw mode for direct key reading"""
        tty.setraw(self.fd)
        # Set minimal timeout
        new_settings = termios.tcgetattr(self.fd)
        new_settings[6][termios.VMIN] = 0  # Non-blocking
        new_settings[6][termios.VTIME] = 0  # No timeout
        termios.tcsetattr(self.fd, termios.TCSADRAIN, new_settings)
    
    def get_key(self):
        """Get a single key press, returns None if no key"""
        try:
            # Check if key is available
            if select.select([sys.stdin], [], [], 0.01)[0]:
                ch = sys.stdin.read(1)
                
                # Check for escape sequences (arrow keys)
                if ch == '\x1b':
                    # Check if more characters are available
                    if select.select([sys.stdin], [], [], 0.01)[0]:
                        next_ch = sys.stdin.read(1)
                        if next_ch == '[':
                            if select.select([sys.stdin], [], [], 0.01)[0]:
                                arrow_ch = sys.stdin.read(1)
                                if arrow_ch == 'A':
                                    return 'up'
                                elif arrow_ch == 'B':
                                    return 'down'
                                else:
                                    # Read any remaining chars to clear buffer
                                    while select.select([sys.stdin], [], [], 0)[0]:
                                        sys.stdin.read(1)
                return ch
            return None
        except Exception as e:
            return None
    
    def cleanup(self):
        """Restore terminal settings"""
        termios.tcsetattr(self.fd, termios.TCSADRAIN, self.old_settings)

def handle_menu_selection():
    """Optimized menu navigation with proper input handling"""
    global _last_selected_index
    selected_index = 0
    _last_selected_index = -1  # Reset when starting menu
    
    print("=== Optimized Menu ===")
    print("UP/DOWN: navigate, ENTER: select, BACKSPACE: return to logo, Q: quit")
    
    # Create input handler
    input_handler = InputHandler()
    
    try:
        # Initial display
        display_menu(selected_index)
        print(f"Selected: {_menu_items[selected_index]}")
        
        while True:
            key = input_handler.get_key()
            
            if key:
                if key == 'up':
                    selected_index = (selected_index - 1) % len(_menu_items)
                    display_menu(selected_index)
                    print(f"↑ {_menu_items[selected_index]}")
                        
                elif key == 'down':
                    selected_index = (selected_index + 1) % len(_menu_items)
                    display_menu(selected_index)
                    print(f"↓ {_menu_items[selected_index]}")
                        
                elif key == '\r' or key == '\n':  # Enter
                    print(f"✓ Executing: {_menu_items[selected_index]}")
                    # Execute functionality
                    if selected_index == 0:
                        print("NEW FILE functionality")
                    elif selected_index == 1:
                        print("OPEN FILE functionality")
                    elif selected_index == 2:
                        print("SETTINGS functionality")
                    elif selected_index == 3:
                        print("CREDITS functionality")
                    break
                        
                elif key == '\x7f' or key == '\x08':  # Backspace
                    print("Returning to logo...")
                    input_handler.cleanup()  # Clean up before switching
                    
                    # Import and run logo module directly
                    import logo
                    logo.display_logo()
                    time.sleep(2)  # Show logo for 2 seconds
                    
                    # Re-initialize input for when we return
                    input_handler = InputHandler()
                    continue
                    
                elif key == 'q' or key == 'Q':
                    print("Quitting menu...")
                    break
                    
                else:
                    print(f"Key pressed: {repr(key)}")
            else:
                # No key pressed, sleep briefly to reduce CPU usage
                time.sleep(0.01)
                
    except KeyboardInterrupt:
        print("\nMenu interrupted")
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Always clean up terminal settings
        input_handler.cleanup()

# Pre-initialize resources on import
_init_resources()

if __name__ == "__main__":
    handle_menu_selection()