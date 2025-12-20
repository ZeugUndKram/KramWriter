import board
import busio
import digitalio
from PIL import Image
import adafruit_sharpmemorydisplay
import os
import time

# Initialize SPI and CS pin (genau wie im Beispiel)
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)  # inverted chip select

# Initialize display
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def display_logo():
    try:
        # Clear display
        display.fill(1)
        display.show()
        
        # Load logo
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")

        if not os.path.exists(logo_path):
            print(f"Logo nicht gefunden: {logo_path}")
            # Zeige Testmuster (wie im Beispiel)
            image = Image.new("1", (display.width, display.height))
            from PIL import ImageDraw
            draw = ImageDraw.Draw(image)
            draw.rectangle((0, 0, display.width, display.height), outline=0, fill=0)
            draw.rectangle((50, 50, 350, 190), outline=1, fill=1)
            display.image(image)
            display.show()
            return False

        # Load and convert image
        logo = Image.open(logo_path).convert("1")
        
        # Create blank image (wie im Beispiel)
        image = Image.new("1", (display.width, display.height))
        
        # Optional: Weißer Hintergrund
        from PIL import ImageDraw
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
        
        # Center logo
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        image.paste(logo, (x, y))
        
        # Display (genau wie im Beispiel)
        display.image(image)
        display.show()
        
        print("Logo angezeigt!")
        return True

    except Exception as e:
        print(f"Fehler: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    print("Starte Display-Test...")
    if display_logo():
        print("Erfolgreich!")
    else:
        print("Logo konnte nicht geladen werden")
    
    print("Display bleibt für 30 Sekunden an...")
    time.sleep(30)