import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os
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


def display_file_menu(selected_index=0, scroll_offset=0):
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
            font = ImageFont.truetype(font_path, 32)
        else:
            print(f"Custom font not found: {font_path}")
            font = ImageFont.load_default()

        # Get list of files from /files/ folder
        files_dir = os.path.join(script_dir, "files")
        if not os.path.exists(files_dir):
            os.makedirs(files_dir)
            print("Created files directory")

        file_list = []
        if os.path.exists(files_dir):
            for item in os.listdir(files_dir):
                item_path = os.path.join(files_dir, item)
                if os.path.isfile(item_path):
                    file_list.append(item)

        file_list.sort()

        if not file_list:
            file_list = ["No files found"]

        # Calculate how many items can fit on screen
        item_height = 40
        max_visible_items = display.height // item_height

        # Ensure scroll_offset is within bounds
        max_scroll = max(0, len(file_list) - max_visible_items)
        scroll_offset = max(0, min(scroll_offset, max_scroll))

        # Get the visible subset of files
        visible_files = file_list[scroll_offset:scroll_offset + max_visible_items]

        # Create display image with white background
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)

        # Calculate starting position for visible items
        start_y = 0  # Start at top since we're scrolling

        # Draw each visible file item centered
        for i, filename in enumerate(visible_files):
            y_position = start_y + (i * item_height)

            # Get text bounding box for proper centering
            bbox = draw.textbbox((0, 0), filename, font=font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]

            x_position = (display.width - text_width) // 2

            # Draw text using custom font
            draw_menu_text(draw, x_position, y_position, filename, font)

            # Draw arrow next to selected item if it's the currently visible selected item
            actual_selected_index = scroll_offset + i
            if actual_selected_index == selected_index and arrow and file_list[0] != "No files found":
                arrow_x = x_position - arrow.width - 15
                arrow_y = y_position + (text_height // 2 - arrow.height // 2) + 8
                image.paste(arrow, (arrow_x, arrow_y))

        # Update display
        display.image(image)
        display.show()

        return file_list, scroll_offset

    except Exception as e:
        print(f"Error displaying file menu: {e}")
        import traceback
        traceback.print_exc()
        return [], 0


def get_key():
    """Get a single key press without requiring Enter"""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    try:
        tty.setraw(sys.stdin.fileno())
        ch = sys.stdin.read(1)
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


def handle_file_selection():
    """Handle file navigation with scroll mechanics"""
    selected_index = 0
    scroll_offset = 0
    file_list, scroll_offset = display_file_menu(selected_index, scroll_offset)

    if not file_list:
        print("No files to display")
        return

    # Calculate how many items can fit on screen
    item_height = 40
    max_visible_items = display.height // item_height

    print("Use UP/DOWN arrows to navigate, ENTER to select, BACKSPACE to return")
    print(f"Files: {len(file_list)}, Visible: {max_visible_items}")

    while True:
        try:
            key = get_key()

            if key == 'up':  # Up arrow
                if selected_index > 0:
                    selected_index -= 1
                    # If selection moves above visible area, scroll up
                    if selected_index < scroll_offset:
                        scroll_offset = selected_index
                file_list, scroll_offset = display_file_menu(selected_index, scroll_offset)
                print(f"↑ Selected: {file_list[selected_index]} ({selected_index + 1}/{len(file_list)})")

            elif key == 'down':  # Down arrow
                if selected_index < len(file_list) - 1:
                    selected_index += 1
                    # If selection moves below visible area, scroll down
                    if selected_index >= scroll_offset + max_visible_items:
                        scroll_offset = selected_index - max_visible_items + 1
                file_list, scroll_offset = display_file_menu(selected_index, scroll_offset)
                print(f"↓ Selected: {file_list[selected_index]} ({selected_index + 1}/{len(file_list)})")

            elif key == '\r' or key == '\n':  # Enter key
                if file_list[0] != "No files found":
                    print(f"✓ Selected file: {file_list[selected_index]}")
                    print(f"File '{file_list[selected_index]}' would be opened here")
                break

            elif key == '\x7f' or key == '\x08':  # Backspace or Delete
                print("Returning...")
                break

            elif key == 'q' or key == 'Q':  # Quit
                print("Quitting file browser...")
                break

            else:
                print(f"Key pressed: {repr(key)} - Use arrow keys, Enter, Backspace, or Q")

        except KeyboardInterrupt:
            print("\nFile browser interrupted")
            break
        except Exception as e:
            print(f"Error handling selection: {e}")
            break


if __name__ == "__main__":
    print("=== File Browser with Scroll ===")
    handle_file_selection()