"""
FAST BMP Image Viewer for Sharp Memory Display
Optimized for speed - preloads and pre-converts images
"""

import board
import busio
import digitalio
import os
import time
from PIL import Image
import adafruit_sharpmemorydisplay

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Image files to display
IMAGE_FILES = [
    "Credits_0.bmp",
    "Learn_0.bmp", 
    "Settings_0.bmp",
    "Write_0.bmp",
    "Zeugtris_0.bmp"
]

class FastImageLoader:
    def __init__(self, display_width, display_height):
        self.display_width = display_width
        self.display_height = display_height
        self.images = []  # Pre-processed images
        self.image_names = []
        
    def preprocess_image(self, image_path):
        """Pre-process an image for fast display - converts once"""
        try:
            # Load image
            img = Image.open(image_path)
            
            # Store original size
            original_size = (img.width, img.height)
            
            # Convert to 1-bit MONOCHROME (fastest for display)
            if img.mode != '1':
                # Convert directly to 1-bit using dithering for better quality
                img = img.convert('1', dither=Image.FLOYDSTEINBERG)
            
            # Calculate centering position
            x_offset = (self.display_width - img.width) // 2
            y_offset = (self.display_height - img.height) // 2
            
            # Create pre-rendered display buffer
            display_buffer = Image.new("1", (self.display_width, self.display_height), 1)
            display_buffer.paste(img, (x_offset, y_offset))
            
            return display_buffer
            
        except Exception as e:
            print(f"Error preprocessing {image_path}: {e}")
            return None
    
    def load_all_images(self, assets_folder):
        """Load and pre-process all images"""
        print("Pre-loading images (this happens once)...")
        start_time = time.time()
        
        for img_file in IMAGE_FILES:
            img_path = os.path.join(assets_folder, img_file)
            if os.path.exists(img_path):
                print(f"  Loading {img_file}...")
                processed = self.preprocess_image(img_path)
                if processed:
                    self.images.append(processed)
                    self.image_names.append(img_file)
                    print(f"    ✓ Pre-processed: {img_file}")
                else:
                    print(f"    ✗ Failed: {img_file}")
            else:
                print(f"  Missing: {img_file}")
        
        load_time = time.time() - start_time
        print(f"Pre-loaded {len(self.images)} images in {load_time:.2f} seconds")
        
        return len(self.images) > 0
    
    def display_fast(self, index):
        """Display a pre-processed image (VERY fast)"""
        if 0 <= index < len(self.images):
            display.image(self.images[index])
            display.show()
            return True
        return False

def main():
    print("FAST BMP Image Viewer")
    print("=" * 40)
    
    # Find assets folder
    assets_folder = None
    for path in ["/assets/", "./assets/", "assets/", "/home/pi/assets/"]:
        if os.path.exists(path):
            assets_folder = path
            print(f"Found assets at: {path}")
            break
    
    if not assets_folder:
        print("ERROR: No assets folder found!")
        return
    
    # Create fast image loader
    loader = FastImageLoader(display.width, display.height)
    
    # Pre-load all images
    if not loader.load_all_images(assets_folder):
        print("No images could be loaded!")
        return
    
    # Display first image
    current_index = 0
    print(f"\nDisplaying image 1 of {len(loader.images)}...")
    
    start_time = time.time()
    loader.display_fast(current_index)
    display_time = time.time() - start_time
    
    print(f"First display took: {display_time * 1000:.1f} ms")
    print(f"Current: {loader.image_names[current_index]}")
    
    print("\nControls:")
    print("  N or → or Enter - Next image")
    print("  P or ← - Previous image")
    print("  Q - Quit")
    print("  Number 1-5 - Jump to specific image")
    print()
    
    # Benchmark variables
    display_count = 1
    total_display_time = display_time
    
    while True:
        try:
            cmd = input(f"Image {current_index + 1}/{len(loader.images)} [N/P/Q/#]: ").strip().lower()
            
            if cmd == 'q':
                print("\nGoodbye!")
                break
            
            elif cmd.isdigit():
                # Jump to specific image by number
                num = int(cmd) - 1
                if 0 <= num < len(loader.images):
                    old_index = current_index
                    current_index = num
                    
                    start = time.time()
                    loader.display_fast(current_index)
                    elapsed = time.time() - start
                    
                    display_count += 1
                    total_display_time += elapsed
                    
                    print(f"↳ Jumped to image {current_index + 1}: {loader.image_names[current_index]}")
                    print(f"  Display time: {elapsed * 1000:.1f} ms")
                else:
                    print(f"Invalid image number. Use 1-{len(loader.images)}")
            
            elif cmd == 'n' or cmd == '':
                # Next image
                old_index = current_index
                current_index = (current_index + 1) % len(loader.images)
                
                start = time.time()
                loader.display_fast(current_index)
                elapsed = time.time() - start
                
                display_count += 1
                total_display_time += elapsed
                
                print(f"↳ Next: {loader.image_names[current_index]}")
                print(f"  Display time: {elapsed * 1000:.1f} ms")
            
            elif cmd == 'p':
                # Previous image
                old_index = current_index
                current_index = (current_index - 1) % len(loader.images)
                
                start = time.time()
                loader.display_fast(current_index)
                elapsed = time.time() - start
                
                display_count += 1
                total_display_time += elapsed
                
                print(f"↳ Previous: {loader.image_names[current_index]}")
                print(f"  Display time: {elapsed * 1000:.1f} ms")
            
            else:
                print("Invalid command. Use: N (next), P (previous), 1-5 (jump), or Q (quit)")
        
        except KeyboardInterrupt:
            print("\n\nExiting...")
            break
        except Exception as e:
            print(f"\nError: {e}")
            break
    
    # Display statistics
    if display_count > 1:
        avg_time = total_display_time / display_count
        print(f"\nPerformance Statistics:")
        print(f"  Total images displayed: {display_count}")
        print(f"  Total display time: {total_display_time:.2f} seconds")
        print(f"  Average display time: {avg_time * 1000:.1f} ms")
        print(f"  Average FPS: {1/avg_time:.1f}")
    
    # Clear display
    display.fill(1)
    display.show()
    print("\nDisplay cleared.")

def test_display_speed():
    """Test raw display speed without image processing"""
    print("\nTesting raw display performance...")
    
    # Create test patterns
    test_images = []
    
    # 1. All white
    img_white = Image.new("1", (display.width, display.height), 1)
    test_images.append(("White", img_white))
    
    # 2. All black
    img_black = Image.new("1", (display.width, display.height), 0)
    test_images.append(("Black", img_black))
    
    # 3. Checkerboard pattern
    checker = Image.new("1", (display.width, display.height), 1)
    draw = ImageDraw.Draw(checker)
    for y in range(0, display.height, 20):
        for x in range(0, display.width, 20):
            if (x // 20 + y // 20) % 2 == 0:
                draw.rectangle([x, y, x+19, y+19], fill=0)
    test_images.append(("Checker", checker))
    
    # Test display speed
    iterations = 10
    print(f"Testing {len(test_images)} patterns for {iterations} iterations each...")
    
    for name, img in test_images:
        start = time.time()
        for i in range(iterations):
            display.image(img)
            display.show()
        elapsed = time.time() - start
        avg_time = elapsed / iterations
        print(f"  {name}: {avg_time * 1000:.1f} ms per frame ({1/avg_time:.1f} FPS)")
    
    # Clear at end
    display.fill(1)
    display.show()

if __name__ == "__main__":
    # Check if PIL has ImageDraw for testing
    try:
        from PIL import ImageDraw
        run_test = input("Run display speed test first? (y/n): ").lower()
        if run_test == 'y':
            test_display_speed()
            print("\n" + "=" * 40 + "\n")
    except:
        pass
    
    main()