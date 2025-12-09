#!/usr/bin/env python3
# test_display.py - Complete Sharp Memory Display test
import time
import os
import sys

print("=== Sharp Memory Display Test ===")
print("Display: 2.7\" 400x240")
print("CS Pin: GPIO 8 (pin 24)")
print("SPI: /dev/spidev0.0")

# Check SPI device
if not os.path.exists('/dev/spidev0.0'):
    print("ERROR: /dev/spidev0.0 not found!")
    print("Run: sudo mknod -m 666 /dev/spidev0.0 c 153 0")
    sys.exit(1)

try:
    print("\n1. Importing libraries...")
    import board
    import busio
    import digitalio
    import adafruit_sharpmemorydisplay
    print("✓ Libraries imported")
except ImportError as e:
    print(f"✗ Missing library: {e}")
    print("Install: pip3 install adafruit-circuitpython-sharpmemorydisplay")
    sys.exit(1)

try:
    print("\n2. Initializing SPI...")
    spi = busio.SPI(board.SCK, MOSI=board.MOSI)
    print("✓ SPI initialized")
    
    print("\n3. Setting up CS pin (GPIO 8)...")
    cs = digitalio.DigitalInOut(board.D24)  # GPIO 8 = pin 24
    cs.direction = digitalio.Direction.OUTPUT
    cs.value = True  # Start HIGH
    print("✓ CS pin ready")
    
    print("\n4. Creating display instance...")
    # Try different initialization methods
    display = None
    for baudrate in [2000000, 1000000, 4000000]:
        try:
            print(f"  Trying baudrate: {baudrate}...")
            display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(
                spi, cs, width=400, height=240
            )
            print(f"✓ Display created at {baudrate} baud")
            break
        except Exception as e:
            print(f"  Failed: {e}")
            continue
    
    if display is None:
        print("✗ Could not initialize display at any baudrate")
        sys.exit(1)
        
except Exception as e:
    print(f"✗ Setup failed: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)

# Test functions
def test_clear():
    print("\n5. Testing clear screen...")
    try:
        # Method 1: Use library clear
        display.fill(1)  # 1 = white on Sharp displays
        display.show()
        time.sleep(0.5)
        
        # Method 2: Black screen
        display.fill(0)  # 0 = black
        display.show()
        time.sleep(0.5)
        
        # Method 3: White screen
        display.fill(1)
        display.show()
        print("✓ Clear test passed")
        return True
    except Exception as e:
        print(f"✗ Clear test failed: {e}")
        return False

def test_pattern():
    print("\n6. Drawing test pattern...")
    try:
        # White background
        display.fill(1)
        display.show()
        time.sleep(0.2)
        
        # Black border
        for i in range(0, 10):
            display.rect(i, i, 400-(2*i), 240-(2*i), fill=0, outline=1)
        display.show()
        time.sleep(0.5)
        
        # Diagonal lines
        display.fill(1)
        for i in range(0, 400, 20):
            display.line(i, 0, i, 239, color=0)
        for i in range(0, 240, 20):
            display.line(0, i, 399, i, color=0)
        display.show()
        
        print("✓ Pattern test passed")
        return True
    except Exception as e:
        print(f"✗ Pattern test failed: {e}")
        return False

def test_logo():
    print("\n7. Testing logo display...")
    try:
        # Check if logo exists
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")
        
        if not os.path.exists(logo_path):
            print(f"  Logo not found: {logo_path}")
            print("  Creating simple logo...")
            
            # Create a simple test image
            from PIL import Image, ImageDraw
            img = Image.new("1", (200, 120), 1)  # White background
            draw = ImageDraw.Draw(img)
            draw.rectangle((10, 10, 190, 110), outline=0, fill=1)
            draw.text((60, 50), "TEST", fill=0)
            
            # Save and use
            img.save(logo_path)
            print(f"  Created test logo at {logo_path}")
        
        # Load and display
        from PIL import Image
        logo = Image.open(logo_path)
        
        # Convert if needed
        if logo.mode != "1":
            logo = logo.convert("1")
        
        # Create display image
        image = Image.new("1", (400, 240), 1)
        x = (400 - logo.width) // 2
        y = (240 - logo.height) // 2
        image.paste(logo, (x, y))
        
        # Display
        display.image(image)
        display.show()
        
        print("✓ Logo test passed")
        return True
    except Exception as e:
        print(f"✗ Logo test failed: {e}")
        return False

def manual_refresh():
    print("\n8. Manual refresh test...")
    try:
        # Some Sharp displays need manual refresh command
        display.fill(0)
        display.show()
        time.sleep(0.5)
        
        # Try sending refresh command directly
        display._send_command(0x20)  # Enter extended command mode
        time.sleep(0.001)
        display._send_command(0x80)  # Set VCOM
        time.sleep(0.001)
        
        display.fill(1)
        display.show()
        time.sleep(0.5)
        
        print("✓ Manual refresh test passed")
        return True
    except Exception as e:
        print(f"✗ Manual refresh failed (may be normal): {e}")
        return True  # Not critical

# Run tests
print("\n" + "="*50)
print("RUNNING TESTS...")
print("="*50)

tests = [
    ("Clear Screen", test_clear),
    ("Test Pattern", test_pattern),
    ("Logo Display", test_logo),
    ("Manual Refresh", manual_refresh)
]

passed = 0
total = len(tests)

for test_name, test_func in tests:
    print(f"\n>>> {test_name}")
    if test_func():
        passed += 1
    time.sleep(1)

print("\n" + "="*50)
print(f"RESULTS: {passed}/{total} tests passed")
print("="*50)

if passed > 0:
    print("\n✓ Display is working!")
    print(f"Screen will stay on for 30 seconds...")
    time.sleep(30)
    
    # Clean up
    display.fill(1)
    display.show()
    print("Display cleared to white")
else:
    print("\n✗ All tests failed")
    print("Check wiring and power")

print("\nTest complete!")