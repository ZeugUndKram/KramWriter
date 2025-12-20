import numpy as np
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import time

# Constants from Sharp display library
_SHARPMEM_BIT_WRITECMD = 0x01
_SHARPMEM_BIT_VCOM = 0x02

class FastSharpDisplay:
    def __init__(self, spi, cs_pin, width=400, height=240):
        self._spi = spi
        self._cs = cs_pin
        self.width = width
        self.height = height
        self._vcom = True
        
        # Pre-compute headers (HUGE performance gain)
        self.headers = np.zeros((height, 2), dtype=np.uint8)
        for i in range(height):
            self.headers[i] = [0, i + 1]
        
        # Display buffer (packed bits: 400/8 = 50 bytes per line)
        self.buffer = np.zeros((height, width // 8), dtype=np.uint8)
    
    def update_from_image(self, pil_image):
        """Convert PIL image to display buffer using NumPy"""
        if pil_image.mode != "1":
            pil_image = pil_image.convert("1", dither=Image.NONE)
        
        # Convert to numpy and pack bits (50x faster than loops)
        img_array = np.asarray(pil_image, dtype=np.uint8)
        self.buffer = np.packbits(img_array, axis=1)
    
    def show(self):
        """Send entire frame in one SPI transaction"""
        # Build complete frame
        frame_list = []
        
        # Command byte with VCOM
        cmd = _SHARPMEM_BIT_WRITECMD
        if self._vcom:
            cmd |= _SHARPMEM_BIT_VCOM
        self._vcom = not self._vcom
        frame_list.append(cmd)
        
        # Add headers and buffer data
        for i in range(self.height):
            frame_list.extend(self.headers[i].tolist())
            frame_list.extend(self.buffer[i].tolist())
        
        # Add tail bytes
        frame_list.extend([0, 0])
        
        # Single SPI write (eliminates 722 small writes)
        while not self._spi.try_lock():
            pass
        self._spi.configure(baudrate=2000000)  # 2MHz
        self._cs.value = True
        self._spi.write(bytearray(frame_list))
        self._cs.value = False
        self._spi.unlock()

# Initialize with fast display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = FastSharpDisplay(spi, scs, 400, 240)

# In your update_arrow_position():
def update_arrow_position(new_selection):
    # ... (same drawing logic) ...
    
    # Fast update:
    display.update_from_image(current_image)
    display.show()