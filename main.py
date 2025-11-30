import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Create image and drawing object
image = Image.new("1", (display.width, display.height))
draw = ImageDraw.Draw(image)

# Clear display to white
draw.rectangle((0, 0, display.width, display.height), outline=255, fill=255)

# Draw test text
draw.text((50, 100), "Hello World!", font=ImageFont.load_default(), fill=0)

# Draw horizontal line from edge to edge at y=224
draw.line((0, 224, display.width-1, 224), fill=0, width=1)

# Update display
display.image(image)
display.show()