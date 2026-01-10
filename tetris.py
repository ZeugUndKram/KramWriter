import random
import time
import curses
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

# Initialize display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

BLACK = 0
WHITE = 255

class FastTetris:
    def __init__(self):
        self.width = 10
        self.height = 20
        self.zoom = 12
        self.game_x = 50
        self.game_y = 30
        
        # Game state
        self.reset()
        
        # Create image buffers
        self.buffer = Image.new("1", (display.width, display.height))
        self.draw = ImageDraw.Draw(self.buffer)
        
        # Pre-calculate positions
        self.cell_positions = []
        for y in range(self.height):
            row = []
            for x in range(self.width):
                row.append((
                    self.game_x + x * self.zoom,
                    self.game_y + y * self.zoom
                ))
            self.cell_positions.append(row)
    
    def reset(self):
        self.score = 0
        self.lines = 0
        self.level = 1
        self.state = "start"
        self.field = [[0 for _ in range(self.width)] for _ in range(self.height)]
        self.current_piece = self.new_piece()
        self.next_piece = self.new_piece()
        self.last_drop = time.time()
        
        # Shape definitions (simplified)
        self.shapes = [
            [  # I
                [(0, 0), (1, 0), (2, 0), (3, 0)],
                [(0, 0), (0, 1), (0, 2), (0, 3)]
            ],
            [  # O
                [(0, 0), (1, 0), (0, 1), (1, 1)]
            ],
            [  # T
                [(1, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (1, 2)],
                [(1, 0), (0, 1), (1, 1), (1, 2)]
            ],
            [  # L
                [(0, 0), (0, 1), (0, 2), (1, 2)],
                [(0, 0), (1, 0), (2, 0), (0, 1)],
                [(0, 0), (1, 0), (1, 1), (1, 2)],
                [(2, 0), (0, 1), (1, 1), (2, 1)]
            ],
            [  # J
                [(1, 0), (1, 1), (0, 2), (1, 2)],
                [(0, 0), (0, 1), (1, 1), (2, 1)],
                [(0, 0), (1, 0), (0, 1), (0, 2)],
                [(0, 0), (1, 0), (2, 0), (2, 1)]
            ],
            [  # S
                [(1, 0), (2, 0), (0, 1), (1, 1)],
                [(0, 0), (0, 1), (1, 1), (1, 2)]
            ],
            [  # Z
                [(0, 0), (1, 0), (1, 1), (2, 1)],
                [(1, 0), (0, 1), (1, 1), (0, 2)]
            ]
        ]
    
    def new_piece(self):
        return {
            'type': random.randint(0, 6),
            'rotation': 0,
            'x': self.width // 2 - 1,
            'y': 0
        }
    
    def get_piece_cells(self, piece):
        shape = self.shapes[piece['type']][piece['rotation']]
        return [(piece['x'] + dx, piece['y'] + dy) for dx, dy in shape]
    
    def valid_position(self, piece):
        for x, y in self.get_piece_cells(piece):
            if (x < 0 or x >= self.width or 
                y >= self.height or 
                (y >= 0 and self.field[y][x])):
                return False
        return True
    
    def lock_piece(self):
        for x, y in self.get_piece_cells(self.current_piece):
            if y >= 0:
                self.field[y][x] = self.current_piece['type'] + 1
        
        # Check for completed lines
        lines_cleared = 0
        y = self.height - 1
        while y >= 0:
            if all(self.field[y]):
                # Move everything above down
                for y2 in range(y, 0, -1):
                    self.field[y2] = self.field[y2-1][:]
                self.field[0] = [0] * self.width
                lines_cleared += 1
            else:
                y -= 1
        
        # Update score
        if lines_cleared:
            self.lines += lines_cleared
            self.score += [40, 100, 300, 1200][min(lines_cleared - 1, 3)]
            self.level = self.lines // 10 + 1
        
        # New piece
        self.current_piece = self.next_piece
        self.next_piece = self.new_piece()
        
        if not self.valid_position(self.current_piece):
            self.state = "gameover"
    
    def move(self, dx, dy):
        piece = self.current_piece.copy()
        piece['x'] += dx
        piece['y'] += dy
        if self.valid_position(piece):
            self.current_piece = piece
            return True
        return False
    
    def rotate(self):
        piece = self.current_piece.copy()
        piece['rotation'] = (piece['rotation'] + 1) % len(self.shapes[piece['type']])
        if self.valid_position(piece):
            self.current_piece = piece
            return True
        return False
    
    def drop(self):
        while self.move(0, 1):
            pass
        self.lock_piece()
    
    def draw_block(self, x, y, color_idx):
        """Draw a block with simple pattern based on color"""
        px, py = self.cell_positions[y][x]
        
        # Draw border
        self.draw.rectangle([px, py, px + self.zoom - 1, py + self.zoom - 1], 
                          outline=WHITE, fill=BLACK)
        
        if color_idx:  # Non-zero means there's a block
            # Simple fill for speed
            self.draw.rectangle([px + 1, py + 1, px + self.zoom - 2, py + self.zoom - 2], 
                              fill=WHITE)
            
            # Add a simple pattern based on block type
            if self.zoom >= 8:
                pattern_type = (color_idx - 1) % 4
                center_x = px + self.zoom // 2
                center_y = py + self.zoom // 2
                
                if pattern_type == 0:  # Dot
                    self.draw.point((center_x, center_y), BLACK)
                elif pattern_type == 1:  # Cross
                    self.draw.line([(center_x-2, center_y), (center_x+2, center_y)], BLACK)
                    self.draw.line([(center_x, center_y-2), (center_x, center_y+2)], BLACK)
                elif pattern_type == 2:  # Diagonal
                    self.draw.line([(px+2, py+2), (px+self.zoom-3, py+self.zoom-3)], BLACK)
                elif pattern_type == 3:  # Other diagonal
                    self.draw.line([(px+self.zoom-3, py+2), (px+2, py+self.zoom-3)], BLACK)
    
    def render(self):
        # Clear buffer
        self.draw.rectangle([0, 0, display.width, display.height], fill=BLACK)
        
        # Draw border
        border_left = self.game_x - 2
        border_top = self.game_y - 2
        border_right = self.game_x + self.width * self.zoom + 2
        border_bottom = self.game_y + self.height * self.zoom + 2
        self.draw.rectangle([border_left, border_top, border_right, border_bottom], 
                          outline=WHITE, fill=BLACK)
        
        # Draw field
        for y in range(self.height):
            for x in range(self.width):
                if self.field[y][x]:
                    self.draw_block(x, y, self.field[y][x])
        
        # Draw current piece
        if self.state == "start":
            for x, y in self.get_piece_cells(self.current_piece):
                if y >= 0:
                    self.draw_block(x, y, self.current_piece['type'] + 1)
        
        # Draw next piece preview
        preview_x = border_right + 20
        preview_y = border_top
        self.draw.text((preview_x, preview_y), "NEXT:", fill=WHITE)
        
        if self.next_piece:
            preview_cells = self.get_piece_cells(self.next_piece)
            min_x = min(x for x, _ in preview_cells)
            max_x = max(x for x, _ in preview_cells)
            min_y = min(y for _, y in preview_cells)
            
            for x, y in preview_cells:
                px = preview_x + (x - min_x) * (self.zoom - 4) + 10
                py = preview_y + (y - min_y) * (self.zoom - 4) + 25
                self.draw.rectangle([px, py, px + self.zoom - 5, py + self.zoom - 5], 
                                  outline=WHITE, fill=WHITE)
        
        # Draw score
        self.draw.text((preview_x, preview_y + 80), f"SCORE:", fill=WHITE)
        self.draw.text((preview_x, preview_y + 100), f"{self.score:06d}", fill=WHITE)
        self.draw.text((preview_x, preview_y + 130), f"LINES: {self.lines:03d}", fill=WHITE)
        self.draw.text((preview_x, preview_y + 160), f"LEVEL: {self.level:02d}", fill=WHITE)
        
        # Draw controls
        self.draw.text((10, display.height - 20), 
                      "←→:MOVE  ↑:ROTATE  ↓:FAST  SPACE:DROP  Q:QUIT", 
                      fill=WHITE)
        
        # Game over
        if self.state == "gameover":
            self.draw.text((display.width//2 - 45, display.height//2 - 20), 
                          "GAME OVER", fill=WHITE)
            self.draw.text((display.width//2 - 60, display.height//2 + 10), 
                          f"SCORE: {self.score}", fill=WHITE)
            self.draw.text((display.width//2 - 75, display.height//2 + 40), 
                          "R TO RESTART", fill=WHITE)
        
        # Update display
        display.image(self.buffer)
        display.show()


def main():
    game = FastTetris()
    
    print("\n" + "=" * 60)
    print("FAST TETRIS - Optimized for Sharp Memory Display")
    print("=" * 60)
    
    # Use simple keyboard input (no curses for maximum speed)
    import sys
    import select
    import termios
    import tty
    
    old_settings = termios.tcgetattr(sys.stdin)
    
    try:
        tty.setraw(sys.stdin.fileno())
        
        last_drop_time = time.time()
        fast_drop = False
        
        while True:
            # Handle input
            if select.select([sys.stdin], [], [], 0)[0]:
                key = sys.stdin.read(1)
                
                if key == 'q':
                    break
                elif key == 'r' and game.state == "gameover":
                    game.reset()
                elif game.state == "start":
                    if key == '\x1b':  # Escape sequence start
                        # Check for arrow keys
                        next_chars = sys.stdin.read(2)
                        if next_chars == '[A':  # Up
                            game.rotate()
                        elif next_chars == '[B':  # Down
                            fast_drop = True
                        elif next_chars == '[C':  # Right
                            game.move(1, 0)
                        elif next_chars == '[D':  # Left
                            game.move(-1, 0)
                    elif key == ' ':  # Space
                        game.drop()
                    elif key == 'w':  # W for rotate
                        game.rotate()
                    elif key == 'a':  # A for left
                        game.move(-1, 0)
                    elif key == 'd':  # D for right
                        game.move(1, 0)
                    elif key == 's':  # S for down
                        fast_drop = True
            else:
                fast_drop = False
            
            # Game logic
            if game.state == "start":
                current_time = time.time()
                drop_speed = max(0.05, 0.5 / game.level)
                if fast_drop:
                    drop_speed /= 10
                
                if current_time - last_drop_time > drop_speed:
                    if not game.move(0, 1):
                        game.lock_piece()
                    last_drop_time = current_time
            
            # Render (always render for smoothness)
            game.render()
            
            # Tiny sleep
            time.sleep(0.001)
            
    except KeyboardInterrupt:
        pass
    finally:
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
        display.fill(1)
        display.show()
        print("\nGame ended.")


if __name__ == "__main__":
    main()