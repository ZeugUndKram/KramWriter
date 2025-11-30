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
        
        # Convert to grayscale and prepare final logo
        if logo.mode != "L":
            logo_grayscale = logo.convert("L")
        else:
            logo_grayscale = logo
            
        # Pre-convert final logo to 1-bit
        final_logo = logo_grayscale.convert("1", dither=Image.NONE)
        
        # Calculate final position to center the logo
        x_final = (display.width - logo_grayscale.size[0]) // 2
        y_final = (display.height - logo_grayscale.size[1]) // 2
        
        # Convert to numpy array for faster processing
        logo_array = np.array(logo_grayscale, dtype=np.uint8)
        
        # Pre-calculate content mask (where logo has actual content)
        content_mask = (logo_array < 250).astype(np.float32)
        
        # Pre-generate noise for all frames for consistent performance
        np.random.seed(42)  # Consistent noise pattern
        noise_frames = []
        frames = 20  # Reduced frames for smoother performance
        for i in range(frames):
            visibility = (i + 1) / frames
            noise = np.random.normal(0, 60 * (1 - visibility), logo_array.shape).astype(np.float32)
            noise_frames.append(noise)
        
        # Animation parameters - optimized for smoothness
        duration = 1.5
        frame_delay = duration / frames
        
        print("Starting optimized fade-in with upward movement...")
        
        # Pre-create base image
        base_image = Image.new("1", (display.width, display.height), 255)
        
        for frame in range(frames + 1):
            start_time = time.monotonic()
            
            if frame == 0:
                # First frame - completely white
                display.image(base_image)
                display.show()
            elif frame == frames:
                # Last frame - full image at final position
                final_image = base_image.copy()
                final_image.paste(final_logo, (x_final, y_final))
                display.image(final_image)
                display.show()
            else:
                # Calculate progress with easing
                progress = frame / frames
                visibility = progress * progress  # Quadratic easing
                
                # Calculate vertical position
                start_y = display.height
                current_y = int(start_y + (y_final - start_y) * progress)
                
                # Create faded version using pre-generated noise
                white_fade = 255 * (1 - visibility)
                faded_array = logo_array.astype(np.float32) * visibility + white_fade
                
                # Apply pre-generated noise only to content areas
                dithered_array = faded_array + (noise_frames[frame - 1] * content_mask)
                dithered_array = np.clip(dithered_array, 0, 255).astype(np.uint8)
                
                # Convert to PIL and then to 1-bit
                temp_logo = Image.fromarray(dithered_array, mode='L')
                temp_logo = temp_logo.convert("1", dither=Image.FLOYDSTEINBERG)
                
                # Paste onto display
                current_image = base_image.copy()
                current_image.paste(temp_logo, (x_final, current_y))
                
                display.image(current_image)
                display.show()
            
            # Consistent timing - account for processing time
            elapsed = time.monotonic() - start_time
            if elapsed < frame_delay:
                time.sleep(frame_delay - elapsed)
        
        print("Animation completed smoothly!")
        return True
        
    except Exception as e:
        print(f"Error displaying logo: {e}")
        import traceback
        traceback.print_exc()
        return False

def display_logo_ultra_fast():
    """Even faster version with pre-rendered frames"""
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
            logo_grayscale = logo.convert("L")
        else:
            logo_grayscale = logo
            
        final_logo = logo_grayscale.convert("1", dither=Image.NONE)
        
        x_final = (display.width - logo_grayscale.size[0]) // 2
        y_final = (display.height - logo_grayscale.size[1]) // 2
        
        # Pre-render all animation frames
        frames = 15  # Even fewer frames for maximum speed
        animation_frames = []
        
        logo_array = np.array(logo_grayscale, dtype=np.uint8)
        content_mask = (logo_array < 250)
        
        base_image = Image.new("1", (display.width, display.height), 255)
        
        print("Pre-rendering frames for ultra-smooth animation...")
        
        for frame in range(frames):
            progress = (frame + 1) / frames
            visibility = progress * progress
            
            # Vertical position
            start_y = display.height
            current_y = int(start_y + (y_final - start_y) * progress)
            
            # Simple threshold-based fade (fastest method)
            threshold = int(255 * (1 - visibility))
            mask = logo_array < threshold
            
            # Create binary image
            binary_array = np.where(mask & content_mask, 0, 255).astype(np.uint8)
            temp_logo = Image.fromarray(binary_array, mode='L').convert("1", dither=Image.NONE)
            
            # Create frame
            frame_image = base_image.copy()
            frame_image.paste(temp_logo, (x_final, current_y))
            animation_frames.append(frame_image)
        
        # Playback pre-rendered frames
        frame_delay = 1.5 / frames
        
        print("Playing ultra-fast animation...")
        for frame_image in animation_frames:
            start_time = time.monotonic()
            display.image(frame_image)
            display.show()
            
            elapsed = time.monotonic() - start_time
            if elapsed < frame_delay:
                time.sleep(frame_delay - elapsed)
        
        # Ensure final frame is perfect
        final_image = base_image.copy()
        final_image.paste(final_logo, (x_final, y_final))
        display.image(final_image)
        display.show()
        
        print("Ultra-fast animation completed!")
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    print("=== Optimized BMP Logo Animation ===")
    print("Choose version:")
    print("1. Optimized smooth version")
    print("2. Ultra-fast pre-rendered")
    
    try:
        choice = input("Enter choice (1-2, default=1): ").strip()
        if choice == "2":
            success = display_logo_ultra_fast()
        else:
            success = display_logo_fade_in()
    except:
        success = display_logo_fade_in()
    
    if success:
        print("Animation completed successfully!")
    else:
        print("Failed to display logo")