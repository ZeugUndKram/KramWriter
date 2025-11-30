import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os
import time

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def display_logo_fade_in():
    try:
        # Get the directory where this script is located
        script_dir = os.path.dirname(os.path.abspath(__file__))
        graphics_dir = os.path.join(script_dir, "assets")
        logo_path = os.path.join(graphics_dir, "logo.bmp")
        
        # Check if logo file exists
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            print("Please create a 'assets' folder with 'logo.bmp'")
            return False
        
        # Load the BMP image
        logo = Image.open(logo_path)
        print(f"Loaded logo: {logo.size[0]}x{logo.size[1]}, mode: {logo.mode}")
        
        # Convert to grayscale for dithering control
        if logo.mode != "L":
            logo_grayscale = logo.convert("L")
        else:
            logo_grayscale = logo
        
        # Calculate position to center the logo
        x = (display.width - logo_grayscale.size[0]) // 2
        y = (display.height - logo_grayscale.size[1]) // 2
        
        # Animation parameters - 2 second fade in
        duration = 2.0  # seconds
        frames = 20     # number of animation frames
        frame_delay = duration / frames
        
        print("Starting fade-in animation...")
        
        for frame in range(frames + 1):
            # Calculate visibility (0 to 1)
            visibility = frame / frames
            
            # Create display image with white background
            image = Image.new("1", (display.width, display.height), 255)
            
            # Apply dithering based on visibility
            # Lower threshold = more black pixels appear
            threshold = int(255 * (1 - visibility))
            
            # Create temporary logo with current threshold
            temp_logo = logo_grayscale.point(lambda p: 0 if p < threshold else 255)
            
            # Convert to 1-bit
            temp_logo = temp_logo.convert("1", dither=Image.NONE)
            
            # Paste onto display image
            image.paste(temp_logo, (x, y))
            
            # Update display
            display.image(image)
            display.show()
            
            time.sleep(frame_delay)
        
        # Ensure final image is perfectly displayed
        final_image = Image.new("1", (display.width, display.height), 255)
        final_logo = logo_grayscale.convert("1", dither=Image.NONE)
        final_image.paste(final_logo, (x, y))
        display.image(final_image)
        display.show()
        
        print("Fade-in animation completed!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False

if __name__ == "__main__":
    print("=== BMP Logo Fade-In ===")
    success = display_logo_fade_in()
    
    if success:
        print("Logo fade-in completed!")
    else:
        print("Failed to display logo")