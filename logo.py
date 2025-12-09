import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os
import time

# Initialize display with CS on GPIO 8
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
cs = digitalio.DigitalInOut(board.D24)  # GPIO 8 for CS
# Note: Sharp displays often need just CS, not SCS for basic models

# CORRECT initialization for 2.7" 400x240 Sharp Memory Display
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(
    spi, 
    cs, 
    width=400, 
    height=240,
    baudrate=2000000  # Try different speeds if needed
)

# IMPORTANT: Sharp display needs explicit clear and refresh
def init_display():
    """Initialize Sharp display with correct sequence"""
    # Clear display memory
    display.fill(1)  # 1 = white on Sharp displays
    display.show()
    time.sleep(0.1)
    
    # Send VCOM toggle (required for Sharp displays)
    display._send_command(0x20)  # Enter extended command mode
    time.sleep(0.001)
    
    print("Display initialized")

def display_logo():
    try:
        # Initialize display first
        init_display()
        time.sleep(0.1)
        
        # Get the directory where this script is located
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")

        # Check if logo file exists
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            # Draw a test pattern instead
            display.fill(1)  # White
            display.rect(50, 50, 300, 140, fill=0)  # Black rectangle
            display.show()
            time.sleep(2)
            return False

        # Load and convert the BMP image
        logo = Image.open(logo_path)
        print(f"Logo size: {logo.size}, mode: {logo.mode}")

        # Convert to 1-bit monochrome - CRITICAL for Sharp display
        # Sharp: 1=white, 0=black (inverted from some displays)
        if logo.mode != "1":
            print("Converting to 1-bit...")
            # Try different conversions
            try:
                # Method 1: Direct conversion
                logo = logo.convert("1", dither=Image.NONE)
            except:
                # Method 2: Via grayscale
                logo = logo.convert("L")
                logo = logo.convert("1")
        
        # Create display image with WHITE background (1=white)
        image = Image.new("1", (display.width, display.height), 1)
        
        # Calculate position to center the logo
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        
        print(f"Placing logo at ({x}, {y})")

        # Paste logo onto display image
        # Sharp display: 0=black, 1=white
        # If your logo is black-on-white, it should paste correctly
        image.paste(logo, (x, y))
        
        # Debug: Save the processed image
        debug_path = os.path.join(script_dir, "debug_output.bmp")
        image.save(debug_path)
        print(f"Debug image saved to: {debug_path}")

        # Update display
        display.image(image)
        display.show()
        
        # Sharp displays need periodic refresh
        time.sleep(0.1)
        display.refresh()  # Send update command

        print("Logo displayed successfully!")
        return True

    except Exception as e:
        print(f"Error displaying logo: {e}")
        import traceback
        traceback.print_exc()
        
        # Try simple test pattern as fallback
        try:
            display.fill(0)  # Black
            display.show()
            time.sleep(0.5)
            display.fill(1)  # White  
            display.show()
            print("Test pattern displayed")
        except:
            print("Could not display anything")
        
        return False


if __name__ == "__main__":
    print("Starting display test...")
    
    # Give SPI time to initialize
    time.sleep(0.5)
    
    # Test 1: Simple pattern
    print("Test 1: Drawing test pattern...")
    try:
        display.fill(1)  # White
        display.rect(10, 10, 380, 220, fill=0)  # Black border
        display.line(0, 0, 399, 239, color=0)   # Diagonal line
        display.line(0, 239, 399, 0, color=0)   # Other diagonal
        display.show()
        time.sleep(2)
        print("Test pattern should be visible")
    except Exception as e:
        print(f"Test pattern failed: {e}")
    
    # Test 2: Logo
    print("\nTest 2: Loading logo...")
    if display_logo():
        print("Logo displayed!")
    else:
        print("Logo failed, but display is working")
    
    # Keep display on
    print("\nDisplay will stay on for 30 seconds...")
    print("Press Enter to continue to menu...")
    input()
    
    # Switch to menu.py
    script_dir = os.path.dirname(os.path.abspath(__file__))
    menu_path = os.path.join(script_dir, "menu.py")
    
    if os.path.exists(menu_path):
        print(f"Launching {menu_path}...")
        exec(open(menu_path).read())
    else:
        print(f"menu.py not found at {menu_path}")