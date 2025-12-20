#!/usr/bin/env python3
"""
Fast Sharp Memory Display Menu System
Optimized for Raspberry Pi with 2.7" Sharp Memory Display
"""

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

# ============================================================================
# FAST SHARP MEMORY DISPLAY DRIVER (OPTIMIZED)
# ============================================================================

_SHARPMEM_BIT_WRITECMD = 0x01
_SHARPMEM_BIT_VCOM = 0x02

class FastSharpMemoryDisplay:
    """Optimized display driver with NumPy conversion and single SPI write"""
    
    def __init__(self, spi, cs_pin, width=400, height=240):
        """
        Initialize display with optimized settings
        
        Args:
            spi: SPI bus instance
            cs_pin: Chip select pin (must be configured as OUTPUT)
            width: Display width in pixels
            height: Display height in pixels
        """
        # Configure CS pin
        cs_pin.direction = digitalio.Direction.OUTPUT
        cs_pin.value = False
        self._cs = cs_pin
        
        self._spi = spi
        self.width = width
        self.height = height
        self._vcom = True
        
        # Calculate line length in bytes
        self.line_bytes = width // 8
        
        # Pre-compute headers for all lines (HUGE performance gain)
        print(f"Pre-computing headers for {height} lines...")
        self._headers = bytearray()
        for line in range(1, height + 1):
            # Format: [0x00, line_address]
            self._headers.extend([0x00, line & 0xFF])
        
        # Initialize buffer
        self.buffer = bytearray(height * self.line_bytes)
        
        print(f"Display initialized: {width}x{height}, buffer: {len(self.buffer)} bytes")
    
    def image(self, pil_image):
        """
        Convert PIL image to display format using NumPy (replaces slow nested loops)
        
        Args:
            pil_image: PIL Image object (mode "1" preferred)
        """
        start_time = time.time()
        
        # Convert to 1-bit if needed
        if pil_image.mode != "1":
            pil_image = pil_image.convert("1", dither=Image.NONE)
        
        # Convert to numpy array (fast)
        img_array = np.array(pil_image, dtype=np.uint8)
        
        # Invert pixels for Sharp display (black=0, white=1 typically needs inversion)
        img_array = ~img_array
        
        # Pack 8 pixels into 1 byte (along width axis)
        packed = np.packbits(img_array, axis=1)
        
        # Flatten and copy to buffer
        self.buffer[:] = packed.flatten().tobytes()
        
        elapsed = (time.time() - start_time) * 1000
        if elapsed > 10:  # Only print if conversion takes significant time
            print(f"Image conversion: {elapsed:.1f}ms")
    
    def show(self):
        """Send entire frame in ONE SPI transaction (replaces hundreds of small writes)"""
        start_time = time.time()
        
        # Toggle VCOM bit (required by Sharp displays)
        cmd_byte = _SHARPMEM_BIT_WRITECMD
        if self._vcom:
            cmd_byte |= _SHARPMEM_BIT_VCOM
        self._vcom = not self._vcom
        
        # Build complete frame in memory
        total_size = 1 + (self.height * 2) + (self.height * self.line_bytes) + 2
        frame = bytearray(total_size)
        pos = 0
        
        # Command byte
        frame[pos] = cmd_byte
        pos += 1
        
        # Headers and data for each line
        for line in range(self.height):
            # Copy header
            frame[pos:pos+2] = self._headers[line*2:line*2+2]
            pos += 2
            
            # Copy pixel data
            data_start = line * self.line_bytes
            frame[pos:pos+self.line_bytes] = self.buffer[data_start:data_start+self.line_bytes]
            pos += self.line_bytes
        
        # Tail bytes (required)
        frame[pos:pos+2] = bytes([0x00, 0x00])
        
        # SINGLE SPI TRANSACTION (critical optimization!)
        while not self._spi.try_lock():
            pass
        
        try:
            # Configure SPI (2MHz is safe for most displays)
            self._spi.configure(baudrate=2000000, phase=0, polarity=0)
            
            # Enable display
            self._cs.value = True
            
            # Write ALL data at once
            self._spi.write(frame)
            
            # Disable display
            self._cs.value = False
        finally:
            self._spi.unlock()
        
        elapsed = (time.time() - start_time) * 1000
        if elapsed > 10:
            print(f"Display update: {elapsed:.1f}ms")
    
    def clear(self):
        """Clear display to white"""
        clear_img = Image.new("1", (self.width, self.height), 255)
        self.image(clear_img)
        self.show()
        print("Display cleared")

# ============================================================================
# MENU SYSTEM
# ============================================================================

class MenuSystem:
    """Fast menu system optimized for Sharp Memory Display"""
    
    def __init__(self, display):
        self.display = display
        self.menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS", "EXIT"]
        self.current_selection = 0
        self.last_selection = -1
        
        # Resources
        self.arrow_image = None
        self.font = None
        self.text_positions = []
        self.base_image = None
        self.current_image = None
        
        # Performance tracking
        self.update_count = 0
        self.start_time = time.time()
    
    def load_resources(self):
        """Load all resources (fonts, images) once"""
        print("Loading resources...")
        script_dir = os.path.dirname(os.path.abspath(__file__))
        
        # Load or create arrow
        arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
        if os.path.exists(arrow_path):
            self.arrow_image = Image.open(arrow_path)
            if self.arrow_image.mode != "1":
                self.arrow_image = self.arrow_image.convert("1", dither=Image.NONE)
            # Ensure arrow is black (0)
            if np.array(self.arrow_image).mean() > 128:
                self.arrow_image = Image.eval(self.arrow_image, lambda x: 255 - x)
            print(f"Loaded arrow: {self.arrow_image.size}")
        else:
            # Create simple arrow
            self.arrow_image = Image.new("1", (16, 16), 255)  # White background
            draw = ImageDraw.Draw(self.arrow_image)
            draw.polygon([(12, 0), (12, 16), (0, 8)], fill=0)  # Black arrow
            print("Created default arrow")
        
        # Load font
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            try:
                self.font = ImageFont.truetype(font_path, 36)
                print(f"Loaded font: {font_path}")
            except:
                print("Failed to load custom font, using default")
                self.font = ImageFont.load_default()
        else:
            print("Font not found, using default")
            self.font = ImageFont.load_default()
    
    def create_base_image(self):
        """Create base image with all menu text (drawn once)"""
        print("Creating base menu image...")
        
        # Create white background
        self.base_image = Image.new("1", (self.display.width, self.display.height), 255)
        draw = ImageDraw.Draw(self.base_image)
        
        # Calculate positions
        item_height = 44
        total_height = len(self.menu_items) * item_height
        start_y = (self.display.height - total_height) // 2
        
        self.text_positions = []
        
        for i, item in enumerate(self.menu_items):
            y = start_y + (i * item_height)
            
            # Get text bounding box
            bbox = draw.textbbox((0, 0), item, font=self.font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]
            
            # Center horizontally
            x = (self.display.width - text_width) // 2
            
            # Draw text (black)
            draw.text((x, y), item, font=self.font, fill=0)
            
            # Calculate arrow position (left of text)
            arrow_x = x - self.arrow_image.width - 10
            arrow_y = y + (text_height // 2 - self.arrow_image.height // 2)
            
            # Store for quick updates
            self.text_positions.append({
                'text': item,
                'x': x,
                'y': y,
                'arrow_x': arrow_x,
                'arrow_y': arrow_y,
                'clear_rect': (
                    max(0, arrow_x - 2),
                    max(0, arrow_y - 2),
                    min(self.display.width, arrow_x + self.arrow_image.width + 2),
                    min(self.display.height, arrow_y + self.arrow_image.height + 2)
                )
            })
        
        print(f"Created base image with {len(self.menu_items)} menu items")
    
    def draw_initial_menu(self):
        """Draw complete menu once"""
        print("Drawing initial menu...")
        
        # Start with base image (text only)
        self.current_image = self.base_image.copy()
        
        # Draw initial arrow
        pos = self.text_positions[self.current_selection]
        self.current_image.paste(self.arrow_image, (pos['arrow_x'], pos['arrow_y']))
        
        # Display
        self.display.image(self.current_image)
        self.display.show()
        
        self.last_selection = self.current_selection
        print("Menu ready")
    
    def update_selection(self, new_selection):
        """Update arrow position (fast partial update)"""
        if new_selection == self.last_selection:
            return
        
        # Get positions
        old_pos = self.text_positions[self.last_selection]
        new_pos = self.text_positions[new_selection]
        
        # Create draw object for modifications
        draw = ImageDraw.Draw(self.current_image)
        
        # Clear old arrow (draw white rectangle)
        draw.rectangle(old_pos['clear_rect'], fill=255, outline=255)
        
        # Draw new arrow
        self.current_image.paste(self.arrow_image, (new_pos['arrow_x'], new_pos['arrow_y']))
        
        # Update display
        self.display.image(self.current_image)
        self.display.show()
        
        # Update state
        self.last_selection = self.current_selection
        self.current_selection = new_selection
        
        # Performance tracking
        self.update_count += 1
        if self.update_count % 10 == 0:
            elapsed = time.time() - self.start_time
            print(f"Updates: {self.update_count}, Avg: {self.update_count/elapsed:.1f} fps")
    
    def handle_selection(self):
        """Handle menu item selection"""
        item = self.menu_items[self.current_selection]
        print(f"\n{'='*60}")
        print(f"SELECTED: {item}")
        print(f"{'='*60}")
        
        # Simulate some action
        time.sleep(0.3)
        
        # Return True to continue, False to exit
        if item == "EXIT":
            return False
        return True

# ============================================================================
# INPUT HANDLER
# ============================================================================

class InputHandler:
    """Non-blocking input handler for keyboard"""
    
    @staticmethod
    def get_key():
        """Get a single keypress (blocking)"""
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        
        try:
            tty.setraw(fd)
            ch = sys.stdin.read(1)
            
            # Check for arrow keys
            if ch == '\x1b':  # Escape sequence
                next_ch = sys.stdin.read(1)
                if next_ch == '[':
                    arrow_ch = sys.stdin.read(1)
                    if arrow_ch == 'A':
                        return 'up'
                    elif arrow_ch == 'B':
                        return 'down'
                    elif arrow_ch == 'C':
                        return 'right'
                    elif arrow_ch == 'D':
                        return 'left'
            
            return ch.lower() if ch.isalpha() else ch
            
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
    
    @staticmethod
    def print_instructions():
        """Print control instructions"""
        print("\n" + "="*60)
        print("SHARP MEMORY DISPLAY MENU SYSTEM")
        print("="*60)
        print("CONTROLS:")
        print("  ↑/↓    : Navigate menu")
        print("  Enter  : Select item")
        print("  Q      : Quit program")
        print("  C      : Clear display")
        print("  R      : Redraw menu")
        print("="*60)

# ============================================================================
# MAIN APPLICATION
# ============================================================================

def main():
    """Main application entry point"""
    print("Starting Sharp Memory Display Menu System...")
    
    try:
        # Initialize hardware
        print("Initializing SPI...")
        spi = busio.SPI(board.SCK, MOSI=board.MOSI)
        
        print("Initializing chip select...")
        scs = digitalio.DigitalInOut(board.D6)
        
        print("Creating display driver...")
        display = FastSharpMemoryDisplay(spi, scs, 400, 240)
        
        # Create menu system
        menu = MenuSystem(display)
        
        # Load resources
        menu.load_resources()
        menu.create_base_image()
        menu.draw_initial_menu()
        
        # Print instructions
        InputHandler.print_instructions()
        print(f"Current: {menu.menu_items[menu.current_selection]}")
        
        # Main loop
        running = True
        while running:
            try:
                key = InputHandler.get_key()
                
                if key == 'up':
                    new_sel = (menu.current_selection - 1) % len(menu.menu_items)
                    menu.update_selection(new_sel)
                    print(f"↑ {menu.menu_items[new_sel]}")
                    
                elif key == 'down':
                    new_sel = (menu.current_selection + 1) % len(menu.menu_items)
                    menu.update_selection(new_sel)
                    print(f"↓ {menu.menu_items[new_sel]}")
                    
                elif key == '\r' or key == '\n':  # Enter
                    if not menu.handle_selection():
                        running = False
                    else:
                        print("Menu ready...")
                        
                elif key == 'c' or key == 'C':
                    print("Clearing display...")
                    display.clear()
                    time.sleep(1)
                    menu.draw_initial_menu()
                    
                elif key == 'r' or key == 'R':
                    print("Redrawing menu...")
                    menu.draw_initial_menu()
                    
                elif key == 'q' or key == 'Q':
                    print("\nQuitting...")
                    running = False
                    
                else:
                    if key not in ['\x1b', '[', 'A', 'B', 'C', 'D']:
                        print(f"Key: {repr(key)}")
                        
            except KeyboardInterrupt:
                print("\nInterrupted by user")
                running = False
            except Exception as e:
                print(f"Error in main loop: {e}")
                import traceback
                traceback.print_exc()
                time.sleep(1)
    
    except Exception as e:
        print(f"Fatal error: {e}")
        import traceback
        traceback.print_exc()
    
    finally:
        # Cleanup
        print("\n" + "="*60)
        print("Cleaning up...")
        try:
            display.clear()
            print("Display cleared")
        except:
            pass
        print("Goodbye!")
        print("="*60)

# ============================================================================
# ALTERNATIVE: SIMPLE TEST IF MAIN MENU DOESN'T WORK
# ============================================================================

def simple_test():
    """Simple test if main menu has issues"""
    print("\n" + "="*60)
    print("RUNNING SIMPLE DISPLAY TEST")
    print("="*60)
    
    try:
        # Initialize
        spi = busio.SPI(board.SCK, MOSI=board.MOSI)
        scs = digitalio.DigitalInOut(board.D6)
        display = FastSharpMemoryDisplay(spi, scs, 400, 240)
        
        # Create test pattern
        img = Image.new("1", (400, 240), 255)
        draw = ImageDraw.Draw(img)
        
        # Draw border
        draw.rectangle((0, 0, 399, 239), outline=0)
        
        # Draw text
        try:
            font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 24)
        except:
            font = ImageFont.load_default()
        
        draw.text((100, 100), "DISPLAY TEST", font=font, fill=0)
        draw.text((120, 130), "SHARP MEMORY", font=font, fill=0)
        draw.text((150, 160), "WORKING!", font=font, fill=0)
        
        # Display with timing
        print("Updating display...")
        start = time.time()
        display.image(img)
        display.show()
        elapsed = (time.time() - start) * 1000
        
        print(f"✓ Display update: {elapsed:.1f}ms")
        print("You should see test pattern on display")
        print("="*60)
        
        # Wait
        time.sleep(3)
        
        # Clear
        display.clear()
        
    except Exception as e:
        print(f"✗ Test failed: {e}")
        import traceback
        traceback.print_exc()
    
    print("Test complete")

# ============================================================================
# ENTRY POINT
# ============================================================================

if __name__ == "__main__":
    # Uncomment to run simple test instead
    # simple_test()
    
    # Run main application
    main()