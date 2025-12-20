import board
import busio
import digitalio
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import os
import time
import sys
import termios
import tty

# Constants from Sharp display library
_SHARPMEM_BIT_WRITECMD = 0x01
_SHARPMEM_BIT_VCOM = 0x02

class FastSharpMemoryDisplay:
    """Optimized display driver based on forum solutions"""
    def __init__(self, spi, cs_pin, width=400, height=240):
        self._spi = spi
        self._cs = cs_pin
        self.width = width
        self.height = height
        self._vcom = True
        
        # Line length in bytes (400 pixels / 8 bits = 50 bytes)
        self.line_bytes = width // 8
        
        # Pre-compute headers ONCE (address bytes for each line)
        self._headers = bytearray()
        for line in range(1, height + 1):
            self._headers.extend([0, line & 0xFF])  # [0, address]
        
        # Buffer to hold pixel data (height * line_bytes)
        self.buffer = bytearray(height * self.line_bytes)
    
    def image(self, pil_image):
        """Fast image conversion using NumPy - replaces display.image()"""
        if pil_image.mode != "1":
            pil_image = pil_image.convert("1", dither=Image.NONE)
        
        # Convert PIL to numpy array and pack bits
        img_array = np.array(pil_image, dtype=np.uint8)
        
        # Invert if needed (Sharp displays often need inverted)
        img_array = ~img_array
        
        # Pack 8 pixels into 1 byte
        packed = np.packbits(img_array, axis=1)
        
        # Flatten and copy to buffer
        self.buffer[:] = packed.flatten().tobytes()
    
    def show(self):
        """Single SPI transaction - replaces display.show()"""
        # Toggle VCOM bit
        cmd_byte = _SHARPMEM_BIT_WRITECMD
        if self._vcom:
            cmd_byte |= _SHARPMEM_BIT_VCOM
        self._vcom = not self._vcom
        
        # Build complete frame in one bytearray
        frame = bytearray()
        frame.append(cmd_byte)  # Command byte
        
        # Add headers and pixel data line by line
        for line in range(self.height):
            # Add header for this line
            frame.extend(self._headers[line*2:line*2+2])
            
            # Add pixel data for this line
            start_idx = line * self.line_bytes
            frame.extend(self.buffer[start_idx:start_idx + self.line_bytes])
        
        # Add tail bytes
        frame.extend([0, 0])
        
        # SINGLE SPI WRITE (critical optimization!)
        while not self._spi.try_lock():
            pass
        try:
            self._spi.configure(baudrate=2000000)  # 2MHz
            self._cs.value = True
            self._spi.write(frame)
            self._cs.value = False
        finally:
            self._spi.unlock()

# ========== YOUR MENU CODE (OPTIMIZED) ==========

# Initialize display with FAST driver
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = FastSharpMemoryDisplay(spi, scs, 400, 240)

# State variables
arrow_image = None
font = None
menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
current_selection = 0
last_selection = -1
text_positions = []
base_image = None
current_image = None

def setup():
    """One-time setup of all resources"""
    global arrow_image, font, text_positions, base_image
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Load arrow
    arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
    if os.path.exists(arrow_path):
        arrow_image = Image.open(arrow_path)
        if arrow_image.mode != "1":
            arrow_image = arrow_image.convert("1", dither=Image.NONE)
        # Ensure arrow is black on white
        if np.array(arrow_image).mean() > 128:
            arrow_image = Image.eval(arrow_image, lambda x: 255 - x)
    else:
        # Create simple arrow
        arrow_image = Image.new("1", (20, 20), 255)
        draw = ImageDraw.Draw(arrow_image)
        draw.polygon([(15, 0), (15, 20), (0, 10)], fill=0)
    
    # Load font
    font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
    if os.path.exists(font_path):
        font = ImageFont.truetype(font_path, 38)
    else:
        print("Warning: Font not found, using default")
        font = ImageFont.load_default()
    
    # Create base image with all text
    base_image = Image.new("1", (display.width, display.height), 255)
    draw = ImageDraw.Draw(base_image)
    
    item_height = 45
    total_height = len(menu_items) * item_height
    start_y = (display.height - total_height) // 2
    
    text_positions = []
    for i, item in enumerate(menu_items):
        y = start_y + (i * item_height)
        bbox = draw.textbbox((0, 0), item, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        x = (display.width - text_width) // 2
        
        # Draw text to base image
        draw.text((x, y), item, font=font, fill=0)
        
        # Store positions for arrows
        arrow_x = x - arrow_image.width - 15
        arrow_y = y + (text_height // 2 - arrow_image.height // 2) + 9
        
        text_positions.append({
            'x': x,
            'y': y,
            'arrow_x': arrow_x,
            'arrow_y': arrow_y,
            'clear_area': (
                max(0, arrow_x - 2),
                max(0, arrow_y - 2),
                min(display.width, arrow_x + arrow_image.width + 2),
                min(display.height, arrow_y + arrow_image.height + 2)
            )
        })

def draw_initial_menu():
    """Draw complete menu once"""
    global current_image
    
    # Start with base image (text only)
    current_image = base_image.copy()
    
    # Draw initial arrow
    pos = text_positions[current_selection]
    current_image.paste(arrow_image, (pos['arrow_x'], pos['arrow_y']))
    
    # Fast display update
    display.image(current_image)
    display.show()

def update_arrow_position(new_selection):
    """Fast update: only move the arrow"""
    global current_selection, last_selection, current_image
    
    if new_selection == last_selection:
        return
    
    # Create draw object for modifications
    draw = ImageDraw.Draw(current_image)
    
    # Clear old arrow (if any)
    if last_selection >= 0:
        old_pos = text_positions[last_selection]
        draw.rectangle(old_pos['clear_area'], fill=255, outline=255)
    
    # Draw new arrow
    new_pos = text_positions[new_selection]
    current_image.paste(arrow_image, (new_pos['arrow_x'], new_pos['arrow_y']))
    
    # OPTIMIZED: Fast display update
    display.image(current_image)
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
    """Main menu loop"""
    global current_selection, last_selection
    
    # Setup once
    print("Setting up display...")
    setup()
    
    # Draw initial menu
    print("Drawing menu...")
    draw_initial_menu()
    last_selection = current_selection
    
    print("\n" + "="*50)
    print("MENU READY")
    print("↑↓ = Navigate, Enter = Select, Backspace = Logo, Q = Quit")
    print(f"Current: {menu_items[current_selection]}")
    print("="*50)
    
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
                print(f"\n{'='*50}")
                print(f"✓ SELECTED: {menu_items[current_selection]}")
                print(f"{'='*50}")
                
                # Add your functionality here
                if current_selection == 0:
                    print("NEW FILE functionality")
                elif current_selection == 1:
                    print("OPEN FILE functionality")
                elif current_selection == 2:
                    print("SETTINGS functionality")
                elif current_selection == 3:
                    print("CREDITS functionality")
                
                time.sleep(0.5)
                print("Menu ready...")
                
            elif key == '\x7f' or key == '\x08':  # Backspace
                print("\nReturning to logo...")
                return  # Exit menu
                
            elif key == 'q' or key == 'Q':
                print("\nQuitting menu...")
                break
                
            else:
                print(f"Unhandled key: {repr(key)}")
                
        except KeyboardInterrupt:
            print("\nMenu cancelled")
            break
        except Exception as e:
            print(f"Error: {e}")
            import traceback
            traceback.print_exc()
            break
    
    # Clear display on exit
    clear_image = Image.new("1", (display.width, display.height), 255)
    display.image(clear_image)
    display.show()

if __name__ == "__main__":
    run_menu()