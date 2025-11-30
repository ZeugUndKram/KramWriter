import board
import busio
import digitalio
from PIL import Image, ImageFilter
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
        
        # Animation parameters - 2 second fade in
        duration = 2.0
        frames = 30  # More frames for smoother animation
        frame_delay = duration / frames
        
        print("Starting fade-in animation with advanced dithering...")
        
        for frame in range(frames + 1):
            # Calculate visibility (0 to 1)
            visibility = frame / frames
            
            # Create display image with white background
            image = Image.new("1", (display.width, display.height), 255)
            
            if visibility == 0:
                # First frame - completely white
                pass
            elif visibility == 1:
                # Last frame - full image
                final_logo = logo_grayscale.convert("1", dither=Image.NONE)
                image.paste(final_logo, (x, y))
            else:
                # Animated frames with dithering
                # Apply visibility by scaling pixel values
                visible_array = logo_array * visibility
                
                # Add some noise to create dithering effect
                noise = np.random.normal(0, 30 * (1 - visibility), logo_array.shape)
                dithered_array = visible_array + noise
                
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

def display_logo_simple_dither():
    """Simpler version using only PIL dithering"""
    try:
        # Get the directory where this script is located
        script_dir = os.path.dirname(os.path.abspath(__file__))
        graphics_dir = os.path.join(script_dir, "assets")
        logo_path = os.path.join(graphics_dir, "logo.bmp")
        
        # Check if logo file exists
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            return False
        
        # Load and convert to grayscale
        logo = Image.open(logo_path)
        if logo.mode != "L":
            logo_grayscale = logo.convert("L")
        else:
            logo_grayscale = logo
        
        # Center position
        x = (display.width - logo_grayscale.size[0]) // 2
        y = (display.height - logo_grayscale.size[1]) // 2
        
        # Animation - 2 seconds
        frames = 20
        frame_delay = 2.0 / frames
        
        print("Starting simple dither fade-in...")
        
        for frame in range(frames + 1):
            visibility = frame / frames
            
            image = Image.new("1", (display.width, display.height), 255)
            
            if visibility == 0:
                # Skip first frame (blank)
                pass
            elif visibility == 1:
                # Final frame - clean conversion
                final_logo = logo_grayscale.convert("1", dither=Image.NONE)
                image.paste(final_logo, (x, y))
            else:
                # Create a temporary image with adjusted brightness
                temp_logo = logo_grayscale.point(lambda p: min(255, p + int(255 * (1 - visibility))))
                
                # Use different dithering methods for variety
                if visibility < 0.3:
                    dither_method = Image.FLOYDSTEINBERG
                elif visibility < 0.7:
                    dither_method = Image.FLOYDSTEINBERG
                else:
                    dither_method = Image.NONE
                
                temp_logo = temp_logo.convert("1", dither=dither_method)
                image.paste(temp_logo, (x, y))
            
            display.image(image)
            display.show()
            time.sleep(frame_delay)
        
        print("Animation completed!")
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

def display_logo_threshold_fade():
    """Fast version using threshold-based fade"""
    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        graphics_dir = os.path.join(script_dir, "assets")
        logo_path = os.path.join(graphics_dir, "logo.bmp")
        
        if not os.path.exists(logo_path):
            print(f"Logo file not found: {logo_path}")
            return False
        
        # Load and prepare logo
        logo = Image.open(logo_path)
        if logo.mode != "L":
            logo = logo.convert("L")
        
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        
        # Convert to numpy for fast processing
        logo_array = np.array(logo)
        
        frames = 25
        frame_delay = 2.0 / frames
        
        print("Starting fast threshold fade...")
        
        for frame in range(frames + 1):
            visibility = frame / frames
            
            image = Image.new("1", (display.width, display.height), 255)
            
            if visibility > 0:
                # Adjust threshold based on visibility
                threshold = int(128 * (1 - visibility))
                
                # Create binary mask
                mask = logo_array > threshold
                
                # Convert to PIL image
                temp_logo = Image.fromarray((mask * 255).astype(np.uint8), mode='L')
                temp_logo = temp_logo.convert("1", dither=Image.NONE)
                
                image.paste(temp_logo, (x, y))
            
            display.image(image)
            display.show()
            time.sleep(frame_delay)
        
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    print("=== Advanced BMP Logo Fade-In ===")
    print("Choose method:")
    print("1. Advanced dithering with noise (recommended)")
    print("2. Simple PIL dithering")
    print("3. Fast threshold fade")
    
    try:
        choice = input("Enter choice (1-3, default=1): ").strip()
        if choice == "2":
            success = display_logo_simple_dither()
        elif choice == "3":
            success = display_logo_threshold_fade()
        else:
            success = display_logo_fade_in()
    except:
        success = display_logo_fade_in()
    
    if success:
        print("Logo animation completed!")
    else:
        print("Failed to display logo")