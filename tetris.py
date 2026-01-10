import pygame
import random
import time
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

# Initialize the Sharp Memory Display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Tetris colors (converted to 1-bit)
BLACK = 0
WHITE = 255
TETRIS_COLORS = [
    (0, 0, 0),         # Black (empty)
    (120, 37, 179),    # Purple
    (100, 179, 179),   # Cyan
    (80, 34, 22),      # Brown
    (80, 134, 22),     # Green
    (180, 34, 22),     # Red
    (180, 34, 122),    # Pink
]

# Convert colors to grayscale for 1-bit display
def color_to_monochrome(color):
    r, g, b = color
    # Convert to grayscale using standard formula
    gray = 0.299 * r + 0.587 * g + 0.114 * b
    # Use threshold to convert to black/white
    return BLACK if gray < 128 else WHITE

# Create monochrome versions of tetris colors
MONO_COLORS = [color_to_monochrome(color) for color in TETRIS_COLORS]

class Figure:
    x = 0
    y = 0

    figures = [
        [[1, 5, 9, 13], [4, 5, 6, 7]],  # I
        [[4, 5, 9, 10], [2, 6, 5, 9]],  # Z
        [[6, 7, 9, 10], [1, 5, 6, 10]], # S
        [[1, 2, 5, 9], [0, 4, 5, 6], [1, 5, 9, 8], [4, 5, 6, 10]], # J
        [[1, 2, 6, 10], [5, 6, 7, 9], [2, 6, 10, 11], [3, 5, 6, 7]], # L
        [[1, 4, 5, 6], [1, 4, 5, 9], [4, 5, 6, 9], [1, 5, 6, 9]], # T
        [[1, 2, 5, 6]], # O
    ]

    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.type = random.randint(0, len(self.figures) - 1)
        self.color = random.randint(1, len(MONO_COLORS) - 1)
        self.rotation = 0

    def image(self):
        return self.figures[self.type][self.rotation]

    def rotate(self):
        self.rotation = (self.rotation + 1) % len(self.figures[self.type])


class Tetris:
    def __init__(self, height, width):
        self.level = 2
        self.score = 0
        self.state = "start"
        self.field = []
        self.height = height
        self.width = width
        self.x = 30  # X offset on display
        self.y = 20  # Y offset on display
        self.zoom = 10  # Block size
        self.figure = None
        
        # Initialize game field
        self.field = []
        for i in range(height):
            new_line = []
            for j in range(width):
                new_line.append(0)
            self.field.append(new_line)
        
        # Create font for score display
        try:
            self.font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 16)
            self.big_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 24)
        except:
            self.font = ImageFont.load_default()
            self.big_font = ImageFont.load_default()

    def new_figure(self):
        self.figure = Figure(self.width // 2 - 2, 0)

    def intersects(self):
        intersection = False
        for i in range(4):
            for j in range(4):
                if i * 4 + j in self.figure.image():
                    if i + self.figure.y > self.height - 1 or \
                            j + self.figure.x > self.width - 1 or \
                            j + self.figure.x < 0 or \
                            self.field[i + self.figure.y][j + self.figure.x] > 0:
                        intersection = True
        return intersection

    def break_lines(self):
        lines = 0
        for i in range(1, self.height):
            zeros = 0
            for j in range(self.width):
                if self.field[i][j] == 0:
                    zeros += 1
            if zeros == 0:
                lines += 1
                for i1 in range(i, 1, -1):
                    for j in range(self.width):
                        self.field[i1][j] = self.field[i1 - 1][j]
        self.score += lines ** 2

    def go_space(self):
        while not self.intersects():
            self.figure.y += 1
        self.figure.y -= 1
        self.freeze()

    def go_down(self):
        self.figure.y += 1
        if self.intersects():
            self.figure.y -= 1
            self.freeze()

    def freeze(self):
        for i in range(4):
            for j in range(4):
                if i * 4 + j in self.figure.image():
                    self.field[i + self.figure.y][j + self.figure.x] = self.figure.color
        self.break_lines()
        self.new_figure()
        if self.intersects():
            self.state = "gameover"

    def go_side(self, dx):
        old_x = self.figure.x
        self.figure.x += dx
        if self.intersects():
            self.figure.x = old_x

    def rotate(self):
        old_rotation = self.figure.rotation
        self.figure.rotate()
        if self.intersects():
            self.figure.rotation = old_rotation

    def draw(self, draw):
        """Draw the game on a PIL ImageDraw object"""
        # Clear display area
        draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=BLACK)
        
        # Draw border around game area
        border_left = self.x - 2
        border_top = self.y - 2
        border_right = self.x + self.width * self.zoom + 2
        border_bottom = self.y + self.height * self.zoom + 2
        
        draw.rectangle(
            (border_left, border_top, border_right, border_bottom),
            outline=WHITE, fill=BLACK
        )
        
        # Draw grid lines
        for i in range(self.height + 1):
            y = self.y + i * self.zoom
            draw.line([(self.x, y), (self.x + self.width * self.zoom, y)], fill=WHITE)
        
        for j in range(self.width + 1):
            x = self.x + j * self.zoom
            draw.line([(x, self.y), (x, self.y + self.height * self.zoom)], fill=WHITE)
        
        # Draw placed blocks
        for i in range(self.height):
            for j in range(self.width):
                if self.field[i][j] > 0:
                    x = self.x + j * self.zoom
                    y = self.y + i * self.zoom
                    color = MONO_COLORS[self.field[i][j]]
                    draw.rectangle([x, y, x + self.zoom - 1, y + self.zoom - 1], 
                                  outline=WHITE, fill=color)
        
        # Draw current falling figure
        if self.figure is not None:
            for i in range(4):
                for j in range(4):
                    p = i * 4 + j
                    if p in self.figure.image():
                        x = self.x + (j + self.figure.x) * self.zoom
                        y = self.y + (i + self.figure.y) * self.zoom
                        color = MONO_COLORS[self.figure.color]
                        draw.rectangle([x, y, x + self.zoom - 1, y + self.zoom - 1], 
                                      outline=WHITE, fill=color)
        
        # Draw score
        score_text = f"Score: {self.score}"
        draw.text((10, display.height - 30), score_text, font=self.font, fill=WHITE)
        
        # Draw level
        level_text = f"Level: {self.level}"
        draw.text((display.width - 100, display.height - 30), level_text, font=self.font, fill=WHITE)
        
        # Draw game over screen
        if self.state == "gameover":
            # Semi-transparent overlay
            draw.rectangle((0, 0, display.width, display.height), outline=BLACK, fill=BLACK)
            
            # Game over text
            game_over_text = "GAME OVER"
            text_bbox = draw.textbbox((0, 0), game_over_text, font=self.big_font)
            text_width = text_bbox[2] - text_bbox[0]
            text_x = (display.width - text_width) // 2
            text_y = display.height // 2 - 40
            draw.text((text_x, text_y), game_over_text, font=self.big_font, fill=WHITE)
            
            # Final score
            final_score = f"Score: {self.score}"
            score_bbox = draw.textbbox((0, 0), final_score, font=self.font)
            score_width = score_bbox[2] - score_bbox[0]
            score_x = (display.width - score_width) // 2
            score_y = text_y + 40
            draw.text((score_x, score_y), final_score, font=self.font, fill=WHITE)
            
            # Restart instructions
            restart_text = "Press UP to restart"
            restart_bbox = draw.textbbox((0, 0), restart_text, font=self.font)
            restart_width = restart_bbox[2] - restart_bbox[0]
            restart_x = (display.width - restart_width) // 2
            restart_y = score_y + 30
            draw.text((restart_x, restart_y), restart_text, font=self.font, fill=WHITE)


def main():
    # Initialize pygame for keyboard input
    pygame.init()
    pygame.display.set_mode((1, 1))  # Minimal window
    
    # Create Tetris game
    game = Tetris(20, 10)
    game.new_figure()
    
    # Game loop variables
    done = False
    fps = 25
    counter = 0
    pressing_down = False
    
    # Control debouncing
    last_key_time = 0
    key_delay = 0.1  # seconds
    
    print("=" * 50)
    print("TETRIS on Sharp Memory Display")
    print("=" * 50)
    print("CONTROLS:")
    print("  UP    - Rotate")
    print("  DOWN  - Speed drop")
    print("  LEFT  - Move left")
    print("  RIGHT - Move right")
    print("  SPACE - Hard drop")
    print("  ESC   - Exit")
    print("=" * 50)
    
    try:
        while not done:
            current_time = time.time()
            
            # Game logic timing
            counter += 1
            if counter > 100000:
                counter = 0
            
            if counter % (fps // game.level // 2) == 0 or pressing_down:
                if game.state == "start":
                    game.go_down()
            
            # Handle keyboard input
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    done = True
                
                if event.type == pygame.KEYDOWN:
                    if current_time - last_key_time > key_delay:
                        if event.key == pygame.K_UP:
                            game.rotate()
                            last_key_time = current_time
                        elif event.key == pygame.K_DOWN:
                            pressing_down = True
                            last_key_time = current_time
                        elif event.key == pygame.K_LEFT:
                            game.go_side(-1)
                            last_key_time = current_time
                        elif event.key == pygame.K_RIGHT:
                            game.go_side(1)
                            last_key_time = current_time
                        elif event.key == pygame.K_SPACE:
                            game.go_space()
                            last_key_time = current_time
                        elif event.key == pygame.K_ESCAPE:
                            done = True
                        elif event.key == pygame.K_r:
                            # Restart game
                            game = Tetris(20, 10)
                            game.new_figure()
                            last_key_time = current_time
                
                if event.type == pygame.KEYUP:
                    if event.key == pygame.K_DOWN:
                        pressing_down = False
            
            # Update display
            image = Image.new("1", (display.width, display.height))
            draw = ImageDraw.Draw(image)
            game.draw(draw)
            display.image(image)
            display.show()
            
            # Small delay to prevent CPU overload
            time.sleep(0.01)
    
    except KeyboardInterrupt:
        print("\nGame interrupted")
    
    except Exception as e:
        print(f"\nError: {e}")
    
    finally:
        pygame.quit()
        print("\nCleaning up...")
        display.fill(1)
        display.show()
        print("Display cleared. Goodbye!")


if __name__ == "__main__":
    main()