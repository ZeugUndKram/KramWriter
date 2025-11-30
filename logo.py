import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os
import time
import numpy as np

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
        
        # Convert to grayscale
        if logo.mode != "L":
            logo_grayscale = logo.convert("L")
        else:
            logo_grayscale = logo
        
        # Calculate position to center the logo
        x = (display.width - logo_grayscale.size[0]) // 2
        y = (display.height - logo_grayscale.size[1]) // 2
        
        # Convert to numpy array for faster processing
        logo_array = np.array(logo_grayscale, dtype=np.float32)
        
        # Create a mask of where the logo actually has content (not pure white)
        # Pure white pixels (255) will be 0 in the mask, darker pixels will be 1
        content_mask = (logo_array < 250).astype(np.float32)
        
        # Animation parameters - 2 second fade in
        duration = 2.0
        frames = 30
        frame_delay = duration / frames
        
        print("Starting fade-in animation from white (logo pixels only)...")
        
        for frame in range(frames + 1):
            # Calculate visibility (0 to 1)
            visibility = frame / frames
            
            # Create display image with white background
            image = Image.new("1", (display.width, display.height), 255)
            
            if visibility == 0:
                # First frame - completely white (no logo)
                pass
            elif visibility == 1:
                # Last frame - full image
                final_logo = logo_grayscale.convert("1", dither=Image.NONE)
                image.paste(final_logo, (x, y))
            else:
                # Start from white and fade to original image
                white_fade = 255 * (1 - visibility)
                
                # Blend between white and original image
                faded_array = logo_array * visibility + white_fade
                
                # Add dithering noise ONLY to areas with logo content
                noise = np.random.normal(0, 50 * (1 - visibility), logo_array.shape)
                # Apply noise only where there's logo content (using the mask)
                dithered_array = faded_array + (noise * content_mask)
                
                # Clip values to valid range
                dithered_array = np.clip(dithered_array, 0, 255)
                
                # Convert back to PIL Image
                temp_logo = Image.fromarray(dithered_array.astype(np.uint8))
                
                # Convert to 1-bit with Floyd-Steinberg dithering
                temp_logo = temp_logo.convert("1", dither=Image.FLOYDSTEINBERG)
                
                # Paste onto display image
                image.paste(temp_logo, (x, y))
            
            # Update display
            display.image(image)
            display.show()
            
            time.sleep(frame_delay)
        
        print("Fade-in animation completed!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    print("=== BMP Logo Fade-In (Logo Pixels Only) ===")
    success = display_logo_fade_in()
    
    if success:
        print("Logo fade-in completed!")
    else:
        print("Failed to display logo")