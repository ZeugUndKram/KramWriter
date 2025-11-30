#!/usr/bin/env python3
import os
import sys
import termios
import tty


def initialize_display():
    """Safely initialize display with error handling"""
    try:
        import board
        import busio
        import digitalio
        from PIL import Image, ImageDraw, ImageFont
        import adafruit_sharpmemorydisplay

        spi = busio.SPI(board.SCK, MOSI=board.MOSI)
        scs = digitalio.DigitalInOut(board.D6)
        display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)
        return display, None
    except Exception as e:
        return None, f"Display initialization failed: {e}"


def display_file_menu(selected_index=0):
    display, error = initialize_display()
    if error:
        print(f"ERROR: {error}")
        return []

    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))

        # Load arrow
        arrow_path = os.path.join(script_dir, "assets", "arrow.bmp")
        arrow = None
        if os.path.exists(arrow_path):
            from PIL import Image
            arrow = Image.open(arrow_path)
            if arrow.mode != "1":
                arrow = arrow.convert("1", dither=Image.NONE)

        # Load font
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            from PIL import ImageFont
            font = ImageFont.truetype(font_path, 32)
        else:
            from PIL import ImageFont
            font = ImageFont.load_default()

        # Get files
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

        from PIL import Image, ImageDraw
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)

        item_height = 40
        total_height = len(file_list) * item_height
        start_y = (display.height - total_height) // 2

        for i, filename in enumerate(file_list):
            y_position = start_y + (i * item_height)
            bbox = draw.textbbox((0, 0), filename, font=font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]
            x_position = (display.width - text_width) // 2

            draw.text((x_position, y_position), filename, font=font, fill=0)

            if i == selected_index and arrow and file_list[0] != "No files found":
                arrow_x = x_position - arrow.width - 15
                arrow_y = y_position + (text_height // 2 - arrow.height // 2) + 10
                image.paste(arrow, (arrow_x, arrow_y))

        display.image(image)
        display.show()
        return file_list

    except Exception as e:
        print(f"ERROR displaying file menu: {e}")
        return []


def get_key():
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
    selected_index = 0
    file_list = display_file_menu(selected_index)

    if not file_list:
        print("No files to display")
        return

    print("Navigation: ↑/↓ arrows=move, Enter=select, Backspace=back")

    while True:
        try:
            key = get_key()

            if key == 'up':
                selected_index = (selected_index - 1) % len(file_list)
                file_list = display_file_menu(selected_index)
                print(f"Selected: {file_list[selected_index]}")
            elif key == 'down':
                selected_index = (selected_index + 1) % len(file_list)
                file_list = display_file_menu(selected_index)
                print(f"Selected: {file_list[selected_index]}")
            elif key in ['\r', '\n']:
                if file_list[0] != "No files found":
                    print(f"Selected file: {file_list[selected_index]}")
                    # File opening functionality would go here
                break
            elif key in ['\x7f', '\x08']:
                print("Returning to menu...")
                break
            elif key in ['q', 'Q']:
                print("Quitting file browser...")
                break
            else:
                print(f"Unknown key: {repr(key)}")

        except KeyboardInterrupt:
            print("\nFile browser interrupted")
            break
        except Exception as e:
            print(f"ERROR in file browser: {e}")
            break


def main():
    print("=== File Browser ===")
    handle_file_selection()


if __name__ == "__main__":
    main()