import board
import busio
import digitalio
import time
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import usb_hid
from adafruit_hid.keyboard import Keyboard
from adafruit_hid.keycode import Keycode

BLACK = 0
WHITE = 255

# Parameters to Change
BORDER = 5
FONTSIZE = 10
LINE_SPACING = 5  # Space between text lines

# Initialize keyboard
keyboard = Keyboard(usb_hid.devices)

spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)  # inverted chip select

display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Initial text values
text_lines = [
    "Hello World!",
    "Welcome!"
]

# Flag to track if space was pressed
space_pressed = False

def update_display():
    """Update the display with current text lines"""
    display.fill(1)
    
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

# Initial display
update_display()

print("Press SPACE to change the first line to 'penis'")
print("Press any other key to exit (or Ctrl+C)")

try:
    while True:
        # Check for keyboard input
        try:
            # Check if space is pressed
            if keyboard.keys:
                for key in keyboard.keys:
                    if key == Keycode.SPACE:
                        if not space_pressed:  # Only change on initial press, not hold
                            text_lines[0] = "penis"
                            update_display()
                            print("Changed first line to 'penis'")
                            space_pressed = True
                    else:
                        print("Exiting...")
                        break
            else:
                space_pressed = False  # Reset when space is released
                
        except Exception as e:
            # Keyboard might not be available or other error
            pass
            
        time.sleep(0.1)  # Small delay to prevent CPU overload
        
except KeyboardInterrupt:
    print("\nProgram interrupted")