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
        
        # Calculate final position to center the logo
        x_final = (display.width - logo_grayscale.size[0]) // 2
        y_final = (display.height - logo_grayscale.size[1]) // 2
        
        # Convert to numpy array for faster processing
        logo_array = np.array(logo_grayscale, dtype=np.float32)
        
        # Create a mask of where the logo actually has content (not pure white)
        content_mask = (logo_array < 250).astype(np.float32)
        
        # Animation parameters - faster fade in (1 second) with upward movement
        duration = 1.5  # Faster - 1.5 seconds total
        frames = 25     # Fewer frames for faster animation
        frame_delay = duration / frames
        
        print("Starting fade-in animation with upward movement...")
        
        for frame in range(frames + 1):
            # Calculate progress (0 to 1)
            progress = frame / frames
            
            # Create display image with white background
            image = Image.new("1", (display.width, display.height), 255)
            
            if progress == 0:
                # First frame - completely white (no logo)
                pass
            elif progress == 1:
                # Last frame - full image at final position
                final_logo = logo_grayscale.convert("1", dither=Image.NONE)
                image.paste(final_logo, (x_final, y_final))
            else:
                # Calculate current vertical position - start from bottom and rise up
                # Start position: logo completely below the screen
                start_y = display.height
                # End position: centered vertically
                end_y = y_final
                # Current position: interpolate between start and end
                current_y = int(start_y + (end_y - start_y) * progress)
                
                # Calculate visibility - faster fade-in (ease-in curve)
                # Use quadratic easing for faster initial appearance
                visibility = progress * progress  # This makes it appear faster initially
                
                # Start from white and fade to original image
                white_fade = 255 * (1 - visibility)
                
                # Blend between white and original image
                faded_array = logo_array * visibility + white_fade
                
                # Add dithering noise ONLY to areas with logo content
                noise = np.random.normal(0, 60 * (1 - visibility), logo_array.shape)
                # Apply noise only where there's logo content (using the mask)
                dithered_array = faded_array + (noise * content_mask)
                
                # Clip values to valid range
                dithered_array = np.clip(dithered_array, 0, 255)
                
                # Convert back to PIL Image
                temp_logo = Image.fromarray(dithered_array.astype(np.uint8))
                
                # Convert to 1-bit with Floyd-Steinberg dithering
                temp_logo = temp_logo.convert("1", dither=Image.FLOYDSTEINBERG)
                
                # Paste onto display image at current position
                image.paste(temp_logo, (x_final, current_y))
            
            # Update display
            display.image(image)
            display.show()
            
            time.sleep(frame_delay)
        
        print("Animation completed!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    print("=== BMP Logo Fade-In with Upward Movement ===")
    success = display_logo_fade_in()
    
    if success:
        print("Logo animation completed!")
    else:
        print("Failed to display logo")