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


def draw_menu_text(draw, x, y, text, font):
    draw.text((x, y), text, font=font, fill=0)


def display_menu(selected_index=0):
    display, error = initialize_display()
    if error:
        print(f"ERROR: {error}")
        return False

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
        else:
            print(f"Warning: Arrow image not found: {arrow_path}")

        # Load font
        font_path = os.path.join(script_dir, "fonts", "BebasNeue-Regular.ttf")
        if os.path.exists(font_path):
            from PIL import ImageFont
            font = ImageFont.truetype(font_path, 38)
        else:
            print(f"Warning: Custom font not found: {font_path}")
            from PIL import ImageFont
            font = ImageFont.load_default()

        menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]

        from PIL import Image, ImageDraw
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)

        item_height = 45
        total_height = len(menu_items) * item_height
        start_y = (display.height - total_height) // 2

        for i, item in enumerate(menu_items):
            y_position = start_y + (i * item_height)
            bbox = draw.textbbox((0, 0), item, font=font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]
            x_position = (display.width - text_width) // 2

            draw_menu_text(draw, x_position, y_position, item, font)

            if i == selected_index and arrow:
                arrow_x = x_position - arrow.width - 15
                arrow_y = y_position + (text_height // 2 - arrow.height // 2) + 10
                image.paste(arrow, (arrow_x, arrow_y))

        display.image(image)
        display.show()
        return True

    except Exception as e:
        print(f"ERROR displaying menu: {e}")
        return False


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


def handle_menu_selection():
    selected_index = 0
    menu_items = ["NEW FILE", "OPEN FILE", "SETTINGS", "CREDITS"]

    if not display_menu(selected_index):
        print("Failed to initialize menu display")
        return

    print("Navigation: ↑/↓ arrows=move, Enter=select, Backspace=back, Q=quit")

    while True:
        try:
            key = get_key()

            if key == 'up':
                selected_index = (selected_index - 1) % len(menu_items)
                display_menu(selected_index)
                print(f"Selected: {menu_items[selected_index]}")
            elif key == 'down':
                selected_index = (selected_index + 1) % len(menu_items)
                display_menu(selected_index)
                print(f"Selected: {menu_items[selected_index]}")
            elif key in ['\r', '\n']:  # Enter
                selected_item = menu_items[selected_index]
                print(f"Selected: {selected_item}")

                if selected_item == "OPEN FILE":
                    script_dir = os.path.dirname(os.path.abspath(__file__))
                    open_path = os.path.join(script_dir, "open.py")
                    if os.path.exists(open_path):
                        print("Launching file browser...")
                        import subprocess
                        subprocess.run([sys.executable, open_path])
                        # Redisplay menu after return
                        display_menu(selected_index)
                    else:
                        print(f"ERROR: open.py not found at {open_path}")
                else:
                    print(f"Functionality for '{selected_item}' not yet implemented")

            elif key in ['\x7f', '\x08']:  # Backspace
                print("Returning to launcher...")
                break
            elif key in ['q', 'Q']:
                print("Quitting menu...")
                break
            else:
                print(f"Unknown key: {repr(key)}")

        except KeyboardInterrupt:
            print("\nMenu interrupted")
            break
        except Exception as e:
            print(f"ERROR in menu: {e}")
            break


def main():
    print("=== Main Menu ===")
    handle_menu_selection()


if __name__ == "__main__":
    main()