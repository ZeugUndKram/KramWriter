import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

BLACK = 0
WHITE = 255

# Parameters to Change
BORDER = 5
FONTSIZE = 20
LINE_SPACING = 5

spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)

display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Initial display with original text
text_lines = [
    "Hello World!",
    "Welcome!"
]

def update_display():
    """Update the display with current text lines"""
    display.fill(1)
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)

    draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=BLACK)
    draw.rectangle(
        (BORDER, BORDER, display.width - BORDER - 1, display.height - BORDER - 1),
        outline=WHITE,
        fill=WHITE,
    )

    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", FONTSIZE)

    line_metrics = []
    for line in text_lines:
        bbox = font.getbbox(line)
        width = bbox[2] - bbox[0]
        height = bbox[3] - bbox[1]
        line_metrics.append({"text": line, "width": width, "height": height})

    font_height = line_metrics[0]["height"]
    total_text_height = (font_height * len(text_lines)) + (LINE_SPACING * (len(text_lines) - 1))
    start_y = display.height // 2 - total_text_height // 2

    for i, metrics in enumerate(line_metrics):
        y_position = start_y + i * (font_height + LINE_SPACING)
        x_position = display.width // 2 - metrics["width"] // 2
        draw.text((x_position, y_position), metrics["text"], font=font, fill=BLACK)

    display.image(image)
    display.show()

# Show initial display
update_display()

print("The first line will change to 'penis' after this display...")

# Wait a moment, then change to "penis"
import time
time.sleep(2)  # Wait 2 seconds

text_lines[0] = "penis"
update_display()

print("Display updated! First line is now 'penis'")