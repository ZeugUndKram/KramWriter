import random
import time
import curses
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

# Initialize the Sharp Memory Display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Colors
BLACK = 255
WHITE = 0

# Game Boy style patterns for each tetromino type
# Each pattern is defined as a 4x4 binary grid for the texture inside each block
PATTERNS = [
    # 0: I-block (cyan) - vertical lines
    [
        [0, 1, 0, 1],
        [0, 1, 0, 1],
        [0, 1, 0, 1],
        [0, 1, 0, 1]
    ],
    # 1: J-block (blue) - corner pattern
    [
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 1, 1],
        [1, 0, 0, 1]
    ],
    # 2: L-block (orange) - opposite corner
    [
        [0, 0, 1, 1],
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [1, 0, 0, 1]
    ],
    # 3: O-block (yellow) - checkerboard
    [
        [1, 0, 1, 0],
        [0, 1, 0, 1],
        [1, 0, 1, 0],
        [0, 1, 0, 1]
    ],
    # 4: S-block (green) - diagonal stripes
    [
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 1, 1],
        [0, 1, 1, 0]
    ],
    # 5: T-block (purple) - T pattern
    [
        [0, 1, 0, 0],
        [1, 1, 1, 1],
        [0, 1, 0, 0],
        [0, 1, 0, 0]
    ],
    # 6: Z-block (red) - opposite diagonal
    [
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 1, 1],
        [1, 0, 0, 1]
    ]
]

class Figure:
    # Tetromino shapes and their rotations
    shapes = [
        # I
        [
            [1, 5, 9, 13],
            [4, 5, 6, 7]
        ],
        # J
        [
            [1, 2, 5, 9],
            [0, 4, 5, 6],
            [1, 5, 9, 8],
            [4, 5, 6, 10]
        ],
        # L
        [
            [1, 2, 6, 10],
            [5, 6, 7, 9],
            [2, 6, 10, 11],
            [3, 5, 6, 7]
        ],
        # O
        [
            [1, 2, 5, 6]
        ],
        # S
        [
            [6, 7, 9, 10],
            [1, 5, 6, 10]
        ],
        # T
        [
            [1, 4, 5, 6],
            [1, 5, 6, 9],
            [4, 5, 6, 9],
            [1, 4, 5, 9]
        ],
        # Z
        [
            [4, 5, 9, 10],
            [2, 6, 5, 9]
        ]
    ]
    
    # Names for each shape (for debugging)
    names = ["I", "J", "L", "O", "S", "T", "Z"]
    
    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.type = random.randint(0, len(self.shapes) - 1)
        self.rotation = 0
        
    def image(self):
        return self.shapes[self.type][self.rotation]
    
    def rotate(self):
        self.rotation = (self.rotation + 1) % len(self.shapes[self.type])
    
    def get_pattern(self):
        """Get the Game Boy pattern for this tetromino type"""
        return PATTERNS[self.type]

class Tetris:
    def __init__(self, height, width):
        self.level = 1
        self.score = 0
        self.lines = 0
        self.state = "start"
        self.height = height
        self.width = width
        self.x = 50  # X offset
        self.y = 30  # Y offset
        self.zoom = 10  # Block size (must be at least 4 for patterns)
        self.figure = None
        self.next_figure = None
        
        # Initialize field
        self.field = [[-1 for _ in range(width)] for _ in range(height)]
        
        # Load fonts
        try:
            self.font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14)
            self.small_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 10)
        except:
            self.font = ImageFont.load_default()
            self.small_font = ImageFont.load_default()
        
        # Create initial figures
        self.new_figure()
        self.new_next_figure()

    def new_figure(self):
        if self.next_figure:
            self.figure = self.next_figure
            self.figure.x = self.width // 2 - 2
            self.figure.y = 0
        else:
            self.figure = Figure(self.width // 2 - 2, 0)
        self.new_next_figure()
    
    def new_next_figure(self):
        self.next_figure = Figure(0, 0)
        # Position for preview display
        self.next_figure.x = 0
        self.next_figure.y = 0

    def intersects(self):
        for i in range(4):
            for j in range(4):
                if i * 4 + j in self.figure.image():
                    if (i + self.figure.y >= self.height or
                        j + self.figure.x >= self.width or
                        j + self.figure.x < 0 or
                        self.field[i + self.figure.y][j + self.figure.x] != -1):
                        return True
        return False

    def freeze(self):
        for i in range(4):
            for j in range(4):
                if i * 4 + j in self.figure.image():
                    self.field[i + self.figure.y][j + self.figure.x] = self.figure.type
        
        # Check for completed lines
        lines_cleared = 0
        lines_to_remove = []
        for i in range(self.height):
            if all(cell != -1 for cell in self.field[i]):
                lines_to_remove.append(i)
        
        # Remove lines
        for line in reversed(lines_to_remove):
            del self.field[line]
            self.field.insert(0, [-1 for _ in range(self.width)])
            lines_cleared += 1
        
        # Update score
        if lines_cleared > 0:
            self.lines += lines_cleared
            # Original Nintendo scoring system
            points = [40, 100, 300, 1200]  # 1, 2, 3, 4 lines
            self.score += points[min(lines_cleared - 1, 3)] * (self.level + 1)
            self.level = self.lines // 10 + 1
        
        # Create new figure
        self.new_figure()
        if self.intersects():
            self.state = "gameover"

    def go_down(self):
        self.figure.y += 1
        if self.intersects():
            self.figure.y -= 1
            self.freeze()

    def hard_drop(self):
        while not self.intersects():
            self.figure.y += 1
        self.figure.y -= 1
        self.freeze()

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
    
    def draw_block(self, draw, x, y, block_type, size=None):
        """Draw a single block with Game Boy texture"""
        if size is None:
            size = self.zoom
        
        # Draw block outline
        draw.rectangle([x, y, x + size - 1, y + size - 1], 
                      outline=WHITE, fill=BLACK)
        
        if block_type >= 0:
            # Get pattern for this block type
            pattern = PATTERNS[block_type]
            
            # Draw pattern inside block
            pattern_size = size - 4  # Leave border
            if pattern_size >= 4:  # Only draw pattern if block is big enough
                for py in range(4):
                    for px in range(4):
                        if pattern[py][px]:
                            px_pos = x + 2 + (px * pattern_size) // 4
                            py_pos = y + 2 + (py * pattern_size) // 4
                            px_end = x + 2 + ((px + 1) * pattern_size) // 4
                            py_end = y + 2 + ((py + 1) * pattern_size) // 4
                            draw.rectangle([px_pos, py_pos, px_end, py_end], 
                                          outline=WHITE, fill=WHITE)
            else:
                # For very small blocks, just fill
                draw.rectangle([x + 1, y + 1, x + size - 2, y + size - 2], 
                              outline=WHITE, fill=WHITE)

    def draw(self, draw):
        """Draw entire game on display"""
        # Clear display
        draw.rectangle((0, 0, display.width, display.height), fill=BLACK)
        
        # Draw game border
        game_left = self.x - 2
        game_top = self.y - 2
        game_right = self.x + self.width * self.zoom + 2
        game_bottom = self.y + self.height * self.zoom + 2
        
        # Main game area
        draw.rectangle([game_left, game_top, game_right, game_bottom], 
                      outline=WHITE, fill=BLACK)
        
        # Draw placed blocks with textures
        for i in range(self.height):
            for j in range(self.width):
                if self.field[i][j] != -1:
                    x = self.x + j * self.zoom
                    y = self.y + i * self.zoom
                    self.draw_block(draw, x, y, self.field[i][j])
        
        # Draw current falling piece
        if self.figure:
            for i in range(4):
                for j in range(4):
                    if i * 4 + j in self.figure.image():
                        x = self.x + (j + self.figure.x) * self.zoom
                        y = self.y + (i + self.figure.y) * self.zoom
                        self.draw_block(draw, x, y, self.figure.type)
        
        # Draw next piece preview
        preview_x = game_right + 20
        preview_y = game_top
        preview_size = self.zoom - 2
        
        draw.text((preview_x, preview_y), "NEXT:", font=self.font, fill=WHITE)
        
        if self.next_figure:
            # Draw mini-grid for next piece
            preview_grid_x = preview_x
            preview_grid_y = preview_y + 25
            
            # Center the piece in the preview
            min_x = min([(idx % 4) for idx in self.next_figure.image()])
            max_x = max([(idx % 4) for idx in self.next_figure.image()])
            min_y = min([(idx // 4) for idx in self.next_figure.image()])
            max_y = max([(idx // 4) for idx in self.next_figure.image()])
            
            piece_width = (max_x - min_x + 1) * preview_size
            piece_height = (max_y - min_y + 1) * preview_size
            
            start_x = preview_grid_x + (40 - piece_width) // 2
            start_y = preview_grid_y + (40 - piece_height) // 2
            
            for idx in self.next_figure.image():
                i = idx // 4
                j = idx % 4
                x = start_x + (j - min_x) * preview_size
                y = start_y + (i - min_y) * preview_size
                self.draw_block(draw, x, y, self.next_figure.type, preview_size)
        
        # Draw score and stats panel
        panel_x = preview_x
        panel_y = preview_y + 80
        
        draw.text((panel_x, panel_y), f"SCORE:", font=self.font, fill=WHITE)
        draw.text((panel_x, panel_y + 20), f"{self.score:06d}", font=self.font, fill=WHITE)
        
        draw.text((panel_x, panel_y + 45), f"LINES:", font=self.font, fill=WHITE)
        draw.text((panel_x, panel_y + 65), f"{self.lines:03d}", font=self.font, fill=WHITE)
        
        draw.text((panel_x, panel_y + 90), f"LEVEL:", font=self.font, fill=WHITE)
        draw.text((panel_x, panel_y + 110), f"{self.level:02d}", font=self.font, fill=WHITE)
        
        # Draw controls
        controls_y = display.height - 40
        draw.text((10, controls_y), "←→:MOVE", font=self.small_font, fill=WHITE)
        draw.text((90, controls_y), "↑:ROTATE", font=self.small_font, fill=WHITE)
        draw.text((170, controls_y), "↓:FAST", font=self.small_font, fill=WHITE)
        draw.text((240, controls_y), "SPACE:DROP", font=self.small_font, fill=WHITE)
        
        # Draw game over screen
        if self.state == "gameover":
            # Semi-transparent overlay
            overlay = Image.new("1", (display.width, display.height), BLACK)
            overlay_draw = ImageDraw.Draw(overlay)
            for y in range(0, display.height, 2):
                for x in range(0, display.width, 2):
                    if (x + y) % 4 == 0:
                        overlay_draw.point((x, y), WHITE)
            
            # Composite with original
            original = Image.new("1", (display.width, display.height), BLACK)
            original_draw = ImageDraw.Draw(original)
            self.draw_game_only(original_draw)
            display.image(Image.eval(Image.blend(original, overlay, 0.5), lambda x: 0 if x < 128 else 255))
            
            # Game over text
            game_over = "GAME OVER"
            draw.text((display.width // 2 - 45, display.height // 2 - 30), 
                     game_over, font=self.font, fill=WHITE)
            
            final_score = f"FINAL SCORE: {self.score}"
            draw.text((display.width // 2 - 70, display.height // 2), 
                     final_score, font=self.font, fill=WHITE)
            
            restart = "PRESS R TO RESTART"
            draw.text((display.width // 2 - 85, display.height // 2 + 30), 
                     restart, font=self.font, fill=WHITE)
    
    def draw_game_only(self, draw):
        """Draw just the game area without UI (for game over effect)"""
        # Draw placed blocks
        for i in range(self.height):
            for j in range(self.width):
                if self.field[i][j] != -1:
                    x = self.x + j * self.zoom
                    y = self.y + i * self.zoom
                    self.draw_block(draw, x, y, self.field[i][j])


def curses_main(stdscr):
    # Setup curses
    curses.curs_set(0)
    stdscr.nodelay(True)
    stdscr.timeout(100)
    
    # Create game
    game = Tetris(20, 10)
    
    print("\n" + "=" * 60)
    print("TETRIS - Game Boy Edition")
    print("=" * 60)
    print("Each block type has a unique Game Boy-style pattern!")
    print("=" * 60)
    print("CONTROLS:")
    print("  Arrow Keys  - Move and rotate")
    print("  Space       - Hard drop")
    print("  R           - Restart (when game over)")
    print("  Q           - Quit")
    print("=" * 60)
    
    # Game loop variables
    drop_counter = 0
    drop_speed = 20  # Frames per drop (higher = slower)
    fast_drop = False
    
    try:
        while True:
            # Handle input
            key = stdscr.getch()
            
            if key != -1:
                if key == curses.KEY_UP:
                    game.rotate()
                elif key == curses.KEY_DOWN:
                    fast_drop = True
                elif key == curses.KEY_LEFT:
                    game.go_side(-1)
                elif key == curses.KEY_RIGHT:
                    game.go_side(1)
                elif key == ord(' '):  # Space bar
                    game.hard_drop()
                elif key == ord('r') or key == ord('R'):
                    if game.state == "gameover":
                        game = Tetris(20, 10)
                elif key == ord('q') or key == ord('Q'):
                    break
            else:
                fast_drop = False
            
            # Auto-drop logic
            drop_counter += 1
            current_drop_speed = drop_speed // (game.level * 2 if fast_drop else game.level)
            
            if drop_counter >= current_drop_speed and game.state == "start":
                game.go_down()
                drop_counter = 0
            
            # Update display
            image = Image.new("1", (display.width, display.height))
            draw = ImageDraw.Draw(image)
            game.draw(draw)
            display.image(image)
            display.show()
            
            # Small delay
            time.sleep(0.01)
            
    except KeyboardInterrupt:
        pass
    
    except Exception as e:
        print(f"Error: {e}")
    
    finally:
        # Clear display
        display.fill(1)
        display.show()


if __name__ == "__main__":
    try:
        curses.wrapper(curses_main)
    except KeyboardInterrupt:
        print("\nGame interrupted")
    finally:
        print("\nThanks for playing!")
        print("Display cleared.")