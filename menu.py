import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os
import time

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def draw_large_text(draw, x, y, text, scale=3):
    """Draw large text by scaling up the default font"""
    # Create a temporary image to draw the text at normal size
    temp_font = ImageFont.load_default()
    
    # Get the size of the text at normal scale
    bbox = draw.textbbox((0, 0), text, font=temp_font)
    normal_width = bbox[2] - bbox[0]
    normal_height = bbox[3] - bbox[1]
    
    # Create a temporary image for the text
    temp_img = Image.new("1", (normal_width, normal_height), 1)  # White background
    temp_draw = ImageDraw.Draw(temp_img)
    temp_draw.text((0, 0), text, font=temp_font, fill=0)  # Black text
    
    # Scale up the image
    scaled_width = normal_width * scale
    scaled_height = normal_height * scale
    scaled_img = temp_img.resize((scaled_width, scaled_height), Image.NEAREST)
    
    # Paste the scaled text onto the main image
    draw.bitmap((x, y), scaled_img, fill=0)

def display_menu():
    try:
        # Create display image with white background
        image = Image.new("1", (display.width, display.height), 255)
        draw = ImageDraw.Draw(image)
        
        # Menu items
        menu_items = [
            "NEW FILE",
            "OPEN FILE", 
            "SETTINGS",
            "CREDITS"
        ]
        
        # Calculate positions for large text
        scale = 4  # Make text 4x bigger than default
        item_height = 60  # Large spacing between items
        total_height = len(menu_items) * item_height
        start_y = (display.height - total_height) // 2
        
        # Draw each menu item centered with large text
        for i, item in enumerate(menu_items):
            y_position = start_y + (i * item_height)
            
            # Estimate width of scaled text (approx 8 pixels per char at default * scale)
            estimated_width = len(item) * 8 * scale
            x_position = (display.width - estimated_width) // 2
            
            # Draw large text
            draw_large_text(draw, x_position, y_position, item, scale=scale)
        
        # Update display
        display.image(image)
        display.show()
        
        print("Menu displayed with large text!")
        print("1. NEW FILE")
        print("2. OPEN FILE") 
        print("3. SETTINGS")
        print("4. CREDITS")
        print("Press backspace to return to logo")
        
        return True
        
    except Exception as e:
        print(f"Error displaying menu: {e}")
        return False

def handle_menu_selection():
    """Wait for user input and handle menu selection"""
    print("\nSelect an option (1-4), 'q' to quit, or backspace to return to logo:")
    
    while True:
        try:
            selection = input().strip().lower()
            
            if selection == '1':
                print("Selected: NEW FILE")
                # Add your new file functionality here
                break
            elif selection == '2':
                print("Selected: OPEN FILE")
                # Add your open file functionality here
                break
            elif selection == '3':
                print("Selected: SETTINGS")
                # Add your settings functionality here
                break
            elif selection == '4':
                print("Selected: CREDITS")
                # Add your credits functionality here
                break
            elif selection == 'q':
                print("Quitting menu...")
                break
            elif selection == '' or selection == '\x08':  # Backspace or empty input
                print("Returning to logo...")
                # Return to logo.py
                script_dir = os.path.dirname(os.path.abspath(__file__))
                logo_path = os.path.join(script_dir, "logo.py")
                
                if os.path.exists(logo_path):
                    print(f"Returning to {logo_path}...")
                    exec(open(logo_path).read())
                    return  # Exit after launching logo.py
                else:
                    print(f"logo.py not found at {logo_path}")
                break
            else:
                print("Invalid selection. Please choose 1-4, 'q' to quit, or backspace to return to logo:")
                
        except KeyboardInterrupt:
            print("\nMenu interrupted")
            break
        except Exception as e:
            print(f"Error handling selection: {e}")
            break

if __name__ == "__main__":
    print("=== Menu ===")
    success = display_menu()
    
    if success:
        handle_menu_selection()
    else:
        print("Failed to display menu")