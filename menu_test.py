import board
import busio
import digitalio
import sys
import select
import termios
import tty
import time
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

BLACK = 0
WHITE = 255

# Parameters to Change
BORDER = 5
FONTSIZE = 30
LINE_SPACING = 5  # Space between text lines

# Initialize SPI and display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)  # inverted chip select

display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Initial text values
text_lines = [
    "Hello World!",
    "Welcome!"
]

def update_display():
    """Update the display with current text lines"""
    # Create blank image for drawing.
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)

    # Draw a black background
    draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=BLACK)

    # Draw a smaller inner rectangle
    draw.rectangle(
        (BORDER, BORDER, display.width - BORDER - 1, display.height - BORDER - 1),
        outline=WHITE,
        fill=WHITE,
    )

    # Load a TTF font.
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", FONTSIZE)

    # Calculate dimensions for each line
    line_metrics = []
    for line in text_lines:
        bbox = font.getbbox(line)
        width = bbox[2] - bbox[0]
        height = bbox[3] - bbox[1]
        line_metrics.append({"text": line, "width": width, "height": height})

    # Assuming all lines have similar height, use the first one
    font_height = line_metrics[0]["height"]

    # Calculate total text block height
    total_text_height = (font_height * len(text_lines)) + (LINE_SPACING * (len(text_lines) - 1))

    # Calculate starting Y position
    start_y = display.height // 2 - total_text_height // 2

    # Draw each line
    for i, metrics in enumerate(line_metrics):
        y_position = start_y + i * (font_height + LINE_SPACING)
        x_position = display.width // 2 - metrics["width"] // 2
        draw.text(
            (x_position, y_position),
            metrics["text"],
            font=font,
            fill=BLACK,
        )

    display.image(image)
    display.show()

def get_key():
    """Get a single keypress without waiting for Enter"""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    try:
        tty.setraw(sys.stdin.fileno())
        ch = sys.stdin.read(1)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
    return ch

# Initial display
update_display()

print("Press SPACE to change the first line to 'penis'")
print("Press 'q' to exit")

try:
    while True:
        # Check for key press
        if select.select([sys.stdin], [], [], 0)[0]:
            key = get_key()
            if key == 'k':  # Space key
                text_lines[0] = "penis"
                update_display()
                print("Changed first line to 'penis'")
            elif key.lower() == 'q':  # Quit
                print("Exiting...")
                break
            elif key == '\x03':  # Ctrl+C
                raise KeyboardInterrupt
        time.sleep(0.1)
        
except KeyboardInterrupt:
    print("\nProgram interrupted")
except Exception as e:
    print(f"Error: {e}")
finally:
    # Reset terminal
    termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, termios.tcgetattr(sys.stdin.fileno()))