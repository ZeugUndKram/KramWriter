import board
import busio
import digitalio
import keyboard  # pip install keyboard
import time
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

BLACK = 0
WHITE = 255

# Parameters to Change
BORDER = 5
FONTSIZE = 24  # Increased for better visibility
LINE_SPACING = 10
NORMAL_TEXT = "Hello World!"
PENIS_TEXT = "penis"

# Initialize SPI and display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)  # inverted chip select
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Track current state
current_top_text = NORMAL_TEXT
space_was_pressed = False

def update_display(top_text, bottom_text="Welcome!"):
    """Update the display with the given text"""
    # Create blank image
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)

    # Draw background
    draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=BLACK)
    draw.rectangle(
        (BORDER, BORDER, display.width - BORDER - 1, display.height - BORDER - 1),
        outline=WHITE,
        fill=WHITE,
    )

    # Load font
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", FONTSIZE)
    except:
        # Fallback to default font
        font = ImageFont.load_default()

    # Calculate text positions
    top_bbox = font.getbbox(top_text)
    bottom_bbox = font.getbbox(bottom_text)
    
    top_width = top_bbox[2] - top_bbox[0]
    top_height = top_bbox[3] - top_bbox[1]
    bottom_width = bottom_bbox[2] - bottom_bbox[0]
    bottom_height = bottom_bbox[3] - bottom_bbox[1]

    # Center both lines
    top_x = display.width // 2 - top_width // 2
    bottom_x = display.width // 2 - bottom_width // 2
    
    # Position lines with spacing
    total_height = top_height + LINE_SPACING + bottom_height
    start_y = display.height // 2 - total_height // 2
    
    # Draw top text
    draw.text((top_x, start_y), top_text, font=font, fill=BLACK)
    
    # Draw bottom text
    draw.text((bottom_x, start_y + top_height + LINE_SPACING), 
              bottom_text, font=font, fill=BLACK)

    # Update display
    display.image(image)
    display.show()

# Initial display
print("=" * 50)
print("PENIS DISPLAY CONTROLLER")
print("=" * 50)
print("INSTRUCTIONS:")
print("- Press SPACE to toggle between 'Hello World!' and 'penis'")
print("- Press ESC to exit")
print("- No need to press Enter!")
print("=" * 50)

update_display(current_top_text)

try:
    while True:
        # Check for space key (toggle text)
        if keyboard.is_pressed('space'):
            if not space_was_pressed:  # Prevent holding key from spamming
                if current_top_text == NORMAL_TEXT:
                    current_top_text = PENIS_TEXT
                    print(f"Text changed to: {PENIS_TEXT}")
                else:
                    current_top_text = NORMAL_TEXT
                    print(f"Text changed to: {NORMAL_TEXT}")
                
                update_display(current_top_text)
                space_was_pressed = True
        else:
            space_was_pressed = False  # Reset when space is released

        # Check for escape key to exit
        if keyboard.is_pressed('esc'):
            print("\nExiting program...")
            break

        # Small delay to prevent CPU overload
        time.sleep(0.05)

except KeyboardInterrupt:
    print("\nProgram interrupted by user")

except Exception as e:
    print(f"\nError: {e}")

finally:
    print("Cleaning up...")
    # Clear display
    display.fill(1)
    display.show()
    print("Display cleared. Goodbye!")