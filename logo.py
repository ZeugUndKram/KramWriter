import board
import busio
import digitalio
from PIL import Image, ImageDraw
import adafruit_sharpmemorydisplay
import os
import time

# Initialize SPI and CS pin
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

def display_logo():
    try:
        display.fill(1)
        display.show()
        
        script_dir = os.path.dirname(os.path.abspath(__file__))
        logo_path = os.path.join(script_dir, "assets", "logo.bmp")

        if not os.path.exists(logo_path):
            print(f"Logo nicht gefunden: {logo_path}")
            return False

        # Load image
        logo = Image.open(logo_path)
        print(f"Original: {logo.size}, Mode: {logo.mode}")
        
        # Convert WITHOUT dithering
        if logo.mode != "1":
            logo = logo.convert("L")  # Zu Grayscale
            # Threshold-basierte Konvertierung (kein Dithering)
            threshold = 128  # Anpassen falls nötig: 100-150
            logo = logo.point(lambda x: 0 if x < threshold else 255, '1')
        
        # Create display image
        image = Image.new("1", (display.width, display.height))
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
        
        # Center and paste
        x = (display.width - logo.size[0]) // 2
        y = (display.height - logo.size[1]) // 2
        image.paste(logo, (x, y))
        
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
    time.sleep(30)