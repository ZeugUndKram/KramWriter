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

def display_logo_with_animation():
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
        
        # Convert to grayscale for dithering
        if logo.mode != "L":
            logo = logo.convert("L")
        
        # Calculate position to center the logo
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        
        # Animation parameters
        steps = 20  # Number of animation steps
        delay = 0.05  # Delay between steps in seconds
        
        print("Starting rising animation...")
        
        for step in range(steps + 1):
            # Create display image with white background
            image = Image.new("1", (display.width, display.height), 255)
            
            # Calculate current vertical position (starts below and rises up)
            # Start with logo completely below, end at final position
            current_y = display.height - int((display.height - y) * step / steps)
            
            # Create a temporary image for dithering effect
            temp_logo = logo.copy()
            
            # Apply dithering based on animation progress
            if step < steps:
                # Calculate visibility factor (0 to 1)
                visibility = step / steps
                
                # Create a threshold mask for dithering
                # We'll use a simple pattern that becomes more solid as animation progresses
                threshold = int(255 * (1 - visibility))
                
                # Apply threshold to create dithering effect
                temp_logo = temp_logo.point(lambda p: 0 if p < threshold else 255)
            
            # Convert to 1-bit for display
            temp_logo = temp_logo.convert("1", dither=Image.NONE)
            
            # Paste the modified logo onto display image
            image.paste(temp_logo, (x, current_y))
            
            # Update display
            display.image(image)
            display.show()
            
            time.sleep(delay)
        
        print("Animation completed!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False

def display_logo_with_simple_rise():
    """Alternative simpler version with just vertical movement"""
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
        
        # Load and convert logo
        logo = Image.open(logo_path)
        print(f"Loaded logo: {logo.size[0]}x{logo.size[1]}, mode: {logo.mode}")
        
        if logo.mode != "1":
            logo = logo.convert("L").convert("1", dither=Image.NONE)
        
        # Calculate position to center the logo
        x = (display.width - logo.size[0]) // 2
        y_final = (display.height - logo.size[1]) // 2
        
        # Animation - simple rise from bottom
        steps = 15
        delay = 0.06
        
        print("Starting simple rising animation...")
        
        for step in range(steps + 1):
            image = Image.new("1", (display.width, display.height), 255)
            
            # Calculate current position (start from bottom, rise to final position)
            progress = step / steps
            current_y = display.height - int((display.height - y_final) * progress)
            
            # For the first few steps, apply dithering to simulate "fading in"
            if step < 3:
                # Create a dithered version by converting with dithering
                temp_logo = logo.convert("L").convert("1", dither=Image.FLOYDSTEINBERG)
                image.paste(temp_logo, (x, current_y))
            else:
                image.paste(logo, (x, current_y))
            
            display.image(image)
            display.show()
            time.sleep(delay)
        
        print("Animation completed!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False

def display_logo_with_scanline_effect():
    """Version with scanline effect - reveals the logo line by line from bottom"""
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
        
        # Load and convert logo
        logo = Image.open(logo_path)
        print(f"Loaded logo: {logo.size[0]}x{logo.size[1]}, mode: {logo.mode}")
        
        if logo.mode != "1":
            logo = logo.convert("L").convert("1", dither=Image.NONE)
        
        # Calculate position to center the logo
        x = (display.width - logo.size[0]) // 2
        y_final = (display.height - logo.size[1]) // 2
        
        # Create final positioned logo
        final_image = Image.new("1", (display.width, display.height), 255)
        final_image.paste(logo, (x, y_final))
        
        # Animation - reveal from bottom with scanlines
        steps = logo.size[1] + 10  # One step per line plus some extra
        delay = 0.02
        
        print("Starting scanline reveal animation...")
        
        for step in range(steps + 1):
            image = Image.new("1", (display.width, display.height), 255)
            
            # Calculate how many lines to show from bottom
            lines_to_show = min(step, logo.size[1])
            
            if lines_to_show > 0:
                # Crop the bottom part of the logo to show
                crop_box = (0, logo.size[1] - lines_to_show, logo.size[0], logo.size[1])
                visible_part = logo.crop(crop_box)
                
                # Paste at correct position
                paste_y = y_final + (logo.size[1] - lines_to_show)
                image.paste(visible_part, (x, paste_y))
            
            display.image(image)
            display.show()
            time.sleep(delay)
        
        print("Animation completed!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        return False

if __name__ == "__main__":
    print("=== Animated BMP Logo Display ===")
    print("Choose animation style:")
    print("1. Dithering rise effect")
    print("2. Simple rise with dithering")
    print("3. Scanline reveal from bottom")
    
    try:
        choice = input("Enter choice (1-3, default=1): ").strip()
        if choice == "2":
            success = display_logo_with_simple_rise()
        elif choice == "3":
            success = display_logo_with_scanline_effect()
        else:
            success = display_logo_with_animation()
    except:
        # Default to first animation if input fails
        success = display_logo_with_animation()
    
    if success:
        print("Logo animation completed successfully!")
    else:
        print("Failed to display logo")