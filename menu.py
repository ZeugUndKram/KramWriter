import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os
import time
import sys
import select

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def draw_large_text(draw, x, y, text, scale=2):
    """Draw large text by scaling up the default font"""
    # Create a temporary image to draw the text at normal size
    temp_font = ImageFont.load_default()
    
    # Get the size of the text at normal scale
    bbox = draw.textbbox((0, 0), text, font=temp_font)
    normal_width = bbox[2] - bbox[0]
    normal_height = bbox[3] - bbox[1]
    
    # Create a temporary image for the text
    temp_img = Image.new("1", (normal_width, normal_height), 1)  # White background
    temp_draw = ImageDraw.Draw(temp_img)
    temp_draw.text((0, 0), text, font=temp_font, fill=0)  # Black text
    
    # Scale up the image
    scaled_width = normal_width * scale
    scaled_height = normal_height * scale
    scaled_img = temp_img.resize((scaled_width, scaled_height), Image.NEAREST)
    
    # Paste the scaled text onto the main image
    draw.bitmap((x, y), scaled_img, fill=0)

def display_menu(selected_index=0):
    try:
        # Load arrow image
        script_dir = os.path.dirname(os.path.abspath(__file__))
        arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
        
        if os.path.exists(arrow_path):
            arrow = Image.open(arrow_path)
            if arrow.mode != "1":
                arrow = arrow.convert("1", dither=Image.NONE)
        else:
            arrow = None
            print(f"Arrow image not found: {arrow_path}")
        
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
        
        # Calculate positions for text
        scale = 2
        item_height = 40
        total_height = len(menu_items) * item_height
        start_y = (display.height - total_height) // 2
        
        # Draw each menu item centered
        for i, item in enumerate(menu_items):
            y_position = start_y + (i * item_height)
            
            # Estimate width of scaled text
            estimated_width = len(item) * 8 * scale
            x_position = (display.width - estimated_width) // 2
            
            # Draw large text
            draw_large_text(draw, x_position, y_position, item, scale=scale)
            
            # Draw arrow next to selected item
            if i == selected_index and arrow:
                arrow_x = x_position - arrow.width - 10  # 10px spacing from text
                arrow_y = y_position + (estimated_width // 2 - arrow.height // 2)
                image.paste(arrow, (arrow_x, arrow_y))
        
        # Update display
        display.image(image)
        display.show()
        
        return True
        
    except Exception as e:
        print(f"Error displaying menu: {e}")
        return False

def get_key():
    """Get a single key press without requiring Enter"""
    if sys.platform == 'win32':
        import msvcrt
        return msvcrt.getch().decode()
    else:
        import termios
        import tty
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        try:
            tty.setraw(sys.stdin.fileno())
            ch = sys.stdin.read(1)
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        return ch

def handle_menu_selection():
    """Handle menu navigation with instant arrow keys"""
    selected_index = 0
    menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]
    
    print("Use UP/DOWN arrows to navigate, ENTER to select, BACKSPACE to return to logo")
    print("Arrow keys should work instantly without pressing Enter")
    
    # Initial display
    display_menu(selected_index)
    
    while True:
        try:
            # Get key press without waiting for Enter
            key = get_key()
            
            if key == '\x1b':  # Escape sequence for arrow keys
                # Check if it's an arrow key
                next_key = get_key()
                if next_key == '[':
                    arrow_key = get_key()
                    if arrow_key == 'A':  # Up arrow
                        selected_index = (selected_index - 1) % len(menu_items)
                        display_menu(selected_index)
                        print(f"↑ Selected: {menu_items[selected_index]}")
                    elif arrow_key == 'B':  # Down arrow
                        selected_index = (selected_index + 1) % len(menu_items)
                        display_menu(selected_index)
                        print(f"↓ Selected: {menu_items[selected_index]}")
            elif key == '\r' or key == '\n':  # Enter key
                print(f"✓ Executing: {menu_items[selected_index]}")
                # Add your functionality here based on selected_index
                if selected_index == 0:
                    print("NEW FILE functionality")
                elif selected_index == 1:
                    print("OPEN FILE functionality")
                elif selected_index == 2:
                    print("SETTINGS functionality")
                elif selected_index == 3:
                    print("CREDITS functionality")
                break
            elif key == '\x7f' or key == '\x08':  # Backspace or Delete
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
            elif key == 'q' or key == 'Q':  # Quit
                print("Quitting menu...")
                break
            else:
                print(f"Key pressed: {repr(key)} - Use arrow keys, Enter, Backspace, or Q")
                
        except KeyboardInterrupt:
            print("\nMenu interrupted")
            break
        except Exception as e:
            print(f"Error handling selection: {e}")
            break

if __name__ == "__main__":
    print("=== Menu with Instant Arrow Keys ===")
    handle_menu_selection()