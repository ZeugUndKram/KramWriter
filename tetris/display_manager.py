import time
from PIL import Image, ImageDraw, ImageFont

class DisplayManager:
    def __init__(self, display, tetris_game):
        self.display = display
        self.game = tetris_game
        self.width = display.width
        self.height = display.height
        
        # Colors
        self.BLACK = 0
        self.WHITE = 255
        
        # Fonts
        try:
            self.font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14)
            self.small_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 12)
        except:
            self.font = ImageFont.load_default()
            self.small_font = ImageFont.load_default()
        
        # Performance tracking
        self.last_update_time = 0
        self.update_interval = 1.0 / 30.0  # 30 FPS for display
        
        # Cached images for static elements
        self.cached_background = None
        self.create_background()
        
        # Dynamic areas
        self.last_score = -1
        self.last_level = -1
        
    def create_background(self):
        """Create cached background with static elements"""
        image = Image.new("1", (self.width, self.height), self.BLACK)
        draw = ImageDraw.Draw(image)
        
        # Draw main game border
        border_left = self.game.x - 1
        border_top = self.game.y - 1
        border_right = self.game.x + self.game.width * self.game.zoom + 1
        border_bottom = self.game.y + self.game.height * self.game.zoom + 1
        
        draw.rectangle(
            (border_left, border_top, border_right, border_bottom),
            outline=self.WHITE, fill=self.BLACK
        )
        
        # Draw preview box
        preview_width = 5 * self.game.preview_zoom
        preview_height = 5 * self.game.preview_zoom
        box_left = self.game.preview_x - 1
        box_top = self.game.preview_y - 1
        box_right = self.game.preview_x + preview_width + 1
        box_bottom = self.game.preview_y + preview_height + 1
        
        draw.rectangle(
            (box_left, box_top, box_right, box_bottom),
            outline=self.WHITE, fill=self.BLACK
        )
        
        # Draw "NEXT" label
        next_text = "NEXT"
        next_width = draw.textlength(next_text, font=self.small_font)
        draw.text(
            (self.game.preview_x + preview_width // 2 - next_width // 2, 
             self.game.preview_y - 20),
            next_text, font=self.small_font, fill=self.WHITE
        )
        
        self.cached_background = image
        
    def draw_game_field(self, draw):
        """Draw the game field (placed blocks)"""
        for i in range(self.game.height):
            # Skip drawing if this line is being cleared and it's in the "off" blink state
            if self.game.state == "clearing" and i in self.game.lines_to_clear and not self.game.line_blink_state:
                # Clear this line
                for j in range(self.game.width):
                    x = self.game.x + j * self.game.zoom
                    y = self.game.y + i * self.game.zoom
                    draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                  outline=self.BLACK, fill=self.BLACK)
                continue
                
            for j in range(self.game.width):
                if self.game.field[i][j] > 0:
                    x = self.game.x + j * self.game.zoom
                    y = self.game.y + i * self.game.zoom
                    draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                  outline=self.WHITE, fill=self.WHITE)
    
    def draw_current_piece(self, draw):
        """Draw the current falling piece"""
        if self.game.figure and self.game.state != "clearing":
            # Clear previous positions
            for i, j in self.game.figure.last_positions:
                x = self.game.x + (j + self.game.figure.last_x) * self.game.zoom
                y = self.game.y + (i + self.game.figure.last_y) * self.game.zoom
                draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                              outline=self.BLACK, fill=self.BLACK)
            
            # Draw new positions
            self.game.figure.last_positions = []
            for i in range(4):
                for j in range(4):
                    if i * 4 + j in self.game.figure.image():
                        x = self.game.x + (j + self.game.figure.x) * self.game.zoom
                        y = self.game.y + (i + self.game.figure.y) * self.game.zoom
                        draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                      outline=self.WHITE, fill=self.WHITE)
                        self.game.figure.last_positions.append((i, j))
            
            # Update last positions for next frame
            self.game.figure.last_x = self.game.figure.x
            self.game.figure.last_y = self.game.figure.y
    
    def draw_preview(self, draw):
        """Draw the next piece preview"""
        if self.game.next_figure:
            # Clear preview area
            preview_width = 5 * self.game.preview_zoom
            preview_height = 5 * self.game.preview_zoom
            draw.rectangle(
                (self.game.preview_x, self.game.preview_y,
                 self.game.preview_x + preview_width,
                 self.game.preview_y + preview_height),
                fill=self.BLACK
            )
            
            # Calculate piece dimensions
            image = self.game.next_figure.image()
            min_x = min([j % 4 for j in image])
            max_x = max([j % 4 for j in image])
            min_y = min([j // 4 for j in image])
            max_y = max([j // 4 for j in image])
            
            piece_width = (max_x - min_x + 1) * self.game.preview_zoom
            piece_height = (max_y - min_y + 1) * self.game.preview_zoom
            
            # Center position
            piece_x = self.game.preview_x + (preview_width - piece_width) // 2
            piece_y = self.game.preview_y + (preview_height - piece_height) // 2
            
            # Draw each block
            for j in image:
                block_x = j % 4 - min_x
                block_y = j // 4 - min_y
                x = piece_x + block_x * self.game.preview_zoom
                y = piece_y + block_y * self.game.preview_zoom
                draw.rectangle(
                    [x, y, x + self.game.preview_zoom - 1, y + self.game.preview_zoom - 1],
                    outline=self.WHITE, fill=self.WHITE
                )
    
    def draw_score_info(self, draw, fps=0, input_lag=0):
        """Draw score, level, and performance info"""
        # Clear info areas
        draw.rectangle((5, 5, 150, 40), fill=self.BLACK)  # Left side
        draw.rectangle((self.width - 155, 5, self.width - 5, 40), fill=self.BLACK)  # Right side
        
        # Draw score (left)
        score_text = f"Score: {self.game.score}"
        draw.text((10, 10), score_text, font=self.small_font, fill=self.WHITE)
        
        # Draw level (right)
        level_text = f"Level: {self.game.level}"
        level_width = draw.textlength(level_text, font=self.small_font)
        draw.text((self.width - level_width - 10, 10), level_text, font=self.small_font, fill=self.WHITE)
        
        # Draw FPS and lag (centered at top)
        if fps > 0:
            fps_text = f"FPS: {fps:.1f}"
            lag_text = f"Lag: {input_lag:.0f}ms"
            
            fps_width = draw.textlength(fps_text, font=self.small_font)
            lag_width = draw.textlength(lag_text, font=self.small_font)
            
            draw.rectangle((self.width//2 - 70, 5, self.width//2 + 70, 25), fill=self.BLACK)
            draw.text((self.width//2 - fps_width - 5, 10), fps_text, font=self.small_font, fill=self.WHITE)
            draw.text((self.width//2 + 5, 10), lag_text, font=self.small_font, fill=self.WHITE)
    
    def draw_game_over(self, draw):
        """Draw game over screen"""
        # Semi-transparent overlay
        for i in range(0, self.height, 2):
            draw.line([(0, i), (self.width, i)], fill=self.WHITE, width=1)
        
        game_over = "GAME OVER"
        final_score = f"Score: {self.game.score}"
        restart = "Press R to restart"
        
        game_over_width = draw.textlength(game_over, font=self.font)
        final_score_width = draw.textlength(final_score, font=self.font)
        restart_width = draw.textlength(restart, font=self.font)
        
        # Draw with black text on white lines
        draw.text((self.width // 2 - game_over_width // 2, self.height // 2 - 40), 
                 game_over, font=self.font, fill=self.BLACK)
        draw.text((self.width // 2 - final_score_width // 2, self.height // 2 - 10), 
                 final_score, font=self.font, fill=self.BLACK)
        draw.text((self.width // 2 - restart_width // 2, self.height // 2 + 20), 
                 restart, font=self.font, fill=self.BLACK)
    
    def update(self, fps=0, input_lag=0):
        """Update the display with current game state"""
        current_time = time.time()
        
        # Rate limit display updates
        if current_time - self.last_update_time < self.update_interval:
            return False
            
        self.last_update_time = current_time
        
        # Start with cached background
        image = self.cached_background.copy()
        draw = ImageDraw.Draw(image)
        
        # Draw dynamic elements
        self.draw_game_field(draw)
        self.draw_current_piece(draw)
        self.draw_preview(draw)
        self.draw_score_info(draw, fps, input_lag)
        
        # Draw game over screen if needed
        if self.game.state == "gameover":
            self.draw_game_over(draw)
        
        # Update display
        try:
            self.display.image(image)
            self.display.show()
            return True
        except Exception as e:
            print(f"Display update error: {e}")
            return False