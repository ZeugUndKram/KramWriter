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


def display_input_screen(filename="", error_message=""):
    try:
        # Load custom font
        script_dir = os.path.dirname(os.path.abspath(__file__))
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            font_large = ImageFont.truetype(font_path, 36)
            font_small = ImageFont.truetype(font_path, 24)
        else:
            print(f"Custom font not found: {font_path}")
            font_large = ImageFont.load_default()
            font_small = ImageFont.load_default()

        # Create display image with white background
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)

        # Display title
        title = "CREATE NEW FILE"
        bbox = draw.textbbox((0, 0), title, font=font_large)
        title_width = bbox[2] - bbox[0]
        title_x = (display.width - title_width) // 2
        draw_menu_text(draw, title_x, 50, title, font_large)

        # Display input field background (rectangle)
        input_width = 350
        input_height = 50
        input_x = (display.width - input_width) // 2
        input_y = 120

        # Draw input field border
        draw.rectangle([input_x, input_y, input_x + input_width, input_y + input_height], outline=0, width=2)

        # Display filename with .txt extension
        display_text = filename + ".txt"
        bbox = draw.textbbox((0, 0), display_text, font=font_large)
        text_width = bbox[2] - bbox[0]

        # Center text in input field (with some padding)
        text_x = input_x + 10
        if text_width < input_width - 20:  # If text fits, center it
            text_x = input_x + (input_width - text_width) // 2

        draw_menu_text(draw, text_x, input_y + 10, display_text, font_large)

        # Display cursor (blinking would be complex, so just show a static indicator)
        cursor_x = text_x + text_width + 2
        draw.line([cursor_x, input_y + 10, cursor_x, input_y + 40], fill=0, width=2)

        # Display instructions
        instructions = "Type filename and press Enter"
        bbox = draw.textbbox((0, 0), instructions, font=font_small)
        instructions_width = bbox[2] - bbox[0]
        instructions_x = (display.width - instructions_width) // 2
        draw_menu_text(draw, instructions_x, 190, instructions, font_small)

        # Display error message if any
        if error_message:
            bbox = draw.textbbox((0, 0), error_message, font=font_small)
            error_width = bbox[2] - bbox[0]
            error_x = (display.width - error_width) // 2
            draw_menu_text(draw, error_x, 220, error_message, font_small)

        # Update display
        display.image(image)
        display.show()

        return True

    except Exception as e:
        print(f"Error displaying input screen: {e}")
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
        return ch
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)


def is_valid_filename(filename):
    """Check if filename contains only valid characters"""
    if not filename:
        return False

    # Check for invalid characters in filenames
    invalid_chars = '<>:"/\\|?*'
    return all(char not in invalid_chars for char in filename)


def handle_file_creation():
    """Handle filename input and file creation"""
    filename = ""
    error_message = ""

    # Initial display
    display_input_screen(filename, error_message)
    print("Type filename (without .txt extension) and press Enter")
    print("Press Backspace to delete, Escape to cancel")

    while True:
        try:
            key = get_key()

            if key == '\r' or key == '\n':  # Enter key
                if not filename.strip():
                    error_message = "Filename cannot be empty"
                    display_input_screen(filename, error_message)
                    print("Error: Filename cannot be empty")
                    continue

                if not is_valid_filename(filename):
                    error_message = "Invalid characters in filename"
                    display_input_screen(filename, error_message)
                    print("Error: Invalid characters in filename")
                    continue

                # Check if file already exists
                script_dir = os.path.dirname(os.path.abspath(__file__))
                files_dir = os.path.join(script_dir, "files")
                full_filename = filename + ".txt"
                file_path = os.path.join(files_dir, full_filename)

                if os.path.exists(file_path):
                    error_message = "File already exists!"
                    display_input_screen(filename, error_message)
                    print(f"Error: File '{full_filename}' already exists")
                else:
                    # Create the file
                    if not os.path.exists(files_dir):
                        os.makedirs(files_dir)

                    with open(file_path, 'w') as f:
                        f.write("")  # Create empty file

                    print(f"✓ Created file: {full_filename}")
                    display_input_screen(filename, "File created successfully!")
                    # Wait a moment to show success message
                    import time
                    time.sleep(2)
                    break

            elif key == '\x7f' or key == '\x08':  # Backspace
                if filename:
                    filename = filename[:-1]
                    error_message = ""
                display_input_screen(filename, error_message)
                print(f"Current input: {filename}")

            elif key == '\x1b':  # Escape key
                print("Cancelled file creation")
                break

            elif key in ['\x7f', '\x08']:  # Backspace (alternative)
                if filename:
                    filename = filename[:-1]
                    error_message = ""
                display_input_screen(filename, error_message)
                print(f"Current input: {filename}")

            elif key == 'q' or key == 'Q':  # Quit
                print("Quitting file creation...")
                break

            elif len(key) == 1 and key.isprintable():  # Regular characters
                # Limit filename length
                if len(filename) < 30:  # Reasonable limit
                    filename += key
                    error_message = ""
                display_input_screen(filename, error_message)
                print(f"Current input: {filename}")

            else:
                print(f"Key pressed: {repr(key)}")

        except KeyboardInterrupt:
            print("\nFile creation interrupted")
            break
        except Exception as e:
            print(f"Error handling input: {e}")
            break


if __name__ == "__main__":
    print("=== Create New File ===")
    handle_file_creation()