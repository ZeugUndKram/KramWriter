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


def draw_menu_text(draw, x, y, text, font):
    """Draw text using the custom font"""
    draw.text((x, y), text, font=font, fill=0)


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

        # Load custom font
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            font = ImageFont.truetype(font_path, 38)
        else:
            print(f"Custom font not found: {font_path}")
            font = ImageFont.load_default()

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
        item_height = 45
        total_height = len(menu_items) * item_height
        start_y = (display.height - total_height) // 2

        # Draw each menu item centered
        for i, item in enumerate(menu_items):
            y_position = start_y + (i * item_height)

            # Get text bounding box for proper centering
            bbox = draw.textbbox((0, 0), item, font=font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]

            x_position = (display.width - text_width) // 2

            # Draw text using custom font
            draw_menu_text(draw, x_position, y_position, item, font)

            # Draw arrow next to selected item
            if i == selected_index and arrow:
                arrow_x = x_position - arrow.width - 15
                arrow_y = y_position + (text_height // 2 - arrow.height // 2) + 10
                image.paste(arrow, (arrow_x, arrow_y))

        # Update display
        display.image(image)
        display.show()

        return True

    except Exception as e:
        print(f"Error displaying menu: {e}")
        import traceback
        traceback.print_exc()
        return False


def get_key():
    """Get a single key press without requiring Enter"""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    try:
        tty.setraw(sys.stdin.fileno())
        ch = sys.stdin.read(1)
        # Check for escape sequences (arrow keys)
        if ch == '\x1b':
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

            if key == 'up':  # Up arrow
                selected_index = (selected_index - 1) % len(menu_items)
                display_menu(selected_index)
                print(f"↑ Selected: {menu_items[selected_index]}")
            elif key == 'down':  # Down arrow
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
    print("=== Menu with Custom Font and Arrow Keys ===")
    handle_menu_selection()