# SPDX-FileCopyrightText: 2024 Sharp Memory Display Text Editor
# SPDX-License-Identifier: MIT

"""
Advanced Text Editor for Sharp Memory Display
Features: ESC menu system, file operations, markdown rendering
"""

import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os
import sys
import select
import termios
import tty
import signal
import json
from datetime import datetime
import re

# Colors
BLACK = 0
WHITE = 255

# Display Parameters
BORDER = 5
FONTSIZE = 16
MAX_LINES = 15
TAB_SIZE = 4

class AdvancedTextEditor:
    def __init__(self):
        # Initialize display
        spi = busio.SPI(board.SCK, MOSI=board.MOSI)
        scs = digitalio.DigitalInOut(board.D6)
        self.display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)
        
        # Clear display
        self.display.fill(1)
        self.display.show()
        
        # Create image and drawing object
        self.image = Image.new("1", (self.display.width, self.display.height))
        self.draw = ImageDraw.Draw(self.image)
        
        # Editor state
        self.lines = [""]
        self.current_line = 0
        self.cursor_pos = 0
        self.scroll_offset = 0
        self.mode = "EDIT"  # EDIT, MENU, COMMAND
        self.filename = None
        self.modified = False
        
        # Menu system
        self.menu_items = ["Save", "Save As", "Open File", "New File", "Settings", "Help", "Quit"]
        self.menu_cursor = 0
        self.submenu_items = []
        self.in_submenu = False
        
        # Display settings
        self.text_color = BLACK
        self.bg_color = WHITE
        self.show_linenumbers = True
        self.show_status = True
        self.wrap_lines = False
        self.syntax_highlighting = True
        
        # Markdown rendering
        self.render_markdown = False
        
        # Font system
        self.fonts = {}
        self.current_font_size = FONTSIZE
        self.load_fonts()
        
        # File history
        self.recent_files = []
        self.max_recent_files = 10
        
        # Configuration
        self.config_file = "text_editor_config.json"
        self.load_config()
        
        # Terminal setup
        self.old_settings = termios.tcgetattr(sys.stdin)
        signal.signal(signal.SIGINT, self.signal_handler)
        
        print("Advanced Text Editor Started!")
        self.show_welcome()

    def show_welcome(self):
        """Display welcome message"""
        print("=== ADVANCED TEXT EDITOR ===")
        print("Press ESC for menu")
        print("Press i to return to editing")
        print("Arrow keys to navigate")

    def load_fonts(self):
        """Load multiple font sizes and styles"""
        font_sizes = [12, 14, 16, 18, 20, 24, 30]
        for size in font_sizes:
            try:
                self.fonts[f"regular_{size}"] = ImageFont.truetype(
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", size
                )
                self.fonts[f"bold_{size}"] = ImageFont.truetype(
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", size
                )
                self.fonts[f"mono_{size}"] = ImageFont.truetype(
                    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", size
                )
            except:
                try:
                    self.fonts[f"regular_{size}"] = ImageFont.truetype(
                        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", size
                    )
                    self.fonts[f"bold_{size}"] = ImageFont.truetype(
                        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf", size
                    )
                except:
                    self.fonts[f"regular_{size}"] = ImageFont.load_default()
        
        self.font = self.fonts[f"regular_{self.current_font_size}"]

    def get_font(self, style="regular", size=None):
        """Get font based on style and size"""
        if size is None:
            size = self.current_font_size
        
        font_key = f"{style}_{size}"
        return self.fonts.get(font_key, self.font)

    def load_config(self):
        """Load editor configuration"""
        try:
            with open(self.config_file, 'r') as f:
                config = json.load(f)
                self.recent_files = config.get('recent_files', [])
                self.current_font_size = config.get('font_size', FONTSIZE)
                self.show_linenumbers = config.get('show_linenumbers', True)
                self.wrap_lines = config.get('wrap_lines', False)
        except FileNotFoundError:
            self.save_config()

    def save_config(self):
        """Save editor configuration"""
        config = {
            'recent_files': self.recent_files[-self.max_recent_files:],
            'font_size': self.current_font_size,
            'show_linenumbers': self.show_linenumbers,
            'wrap_lines': self.wrap_lines
        }
        try:
            with open(self.config_file, 'w') as f:
                json.dump(config, f, indent=2)
        except Exception as e:
            print(f"Config save error: {e}")

    def add_recent_file(self, filename):
        """Add file to recent files list"""
        if filename in self.recent_files:
            self.recent_files.remove(filename)
        self.recent_files.insert(0, filename)
        self.recent_files = self.recent_files[:self.max_recent_files]
        self.save_config()

    def setup_terminal(self):
        """Set up terminal for non-blocking input"""
        tty.setraw(sys.stdin.fileno())

    def get_char(self):
        """Get a single character from stdin without blocking"""
        if select.select([sys.stdin], [], [], 0) == ([sys.stdin], [], []):
            return sys.stdin.read(1)
        return None

    def signal_handler(self, signum, frame):
        """Handle Ctrl+C gracefully"""
        self.cleanup()
        print("\nExiting...")
        sys.exit(0)

    def cleanup(self):
        """Restore terminal settings and clear display"""
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, self.old_settings)
        self.display.fill(1)
        self.display.show()

    # FILE OPERATIONS
    def open_file(self, filename):
        """Open a text file"""
        try:
            with open(filename, 'r') as f:
                self.lines = f.read().splitlines()
                if not self.lines:  # Handle empty files
                    self.lines = [""]
            
            self.current_line = 0
            self.cursor_pos = 0
            self.scroll_offset = 0
            self.filename = filename
            self.modified = False
            self.add_recent_file(filename)
            print(f"Opened: {filename}")
            return True
            
        except Exception as e:
            print(f"Error opening file: {e}")
            return False

    def save_file(self, filename=None):
        """Save current text to file"""
        if filename is None:
            filename = self.filename
        if filename is None:
            return False
        
        try:
            with open(filename, 'w') as f:
                f.write('\n'.join(self.lines))
            
            self.filename = filename
            self.modified = False
            self.add_recent_file(filename)
            print(f"Saved: {filename}")
            return True
            
        except Exception as e:
            print(f"Error saving file: {e}")
            return False

    def save_file_as(self):
        """Save file with new name"""
        filename = self.text_input("Save as: ")
        if filename:
            return self.save_file(filename)
        return False

    def list_files(self, directory="."):
        """List text files in directory"""
        try:
            files = [f for f in os.listdir(directory) 
                    if f.endswith(('.txt', '.md', '.py', '.json'))]
            return sorted(files)
        except Exception as e:
            print(f"Error listing files: {e}")
            return []

    # TEXT EDITING FUNCTIONS
    def insert_char(self, char):
        """Insert character at cursor position"""
        if char == '\t':
            char = ' ' * TAB_SIZE
        
        line = self.lines[self.current_line]
        self.lines[self.current_line] = line[:self.cursor_pos] + char + line[self.cursor_pos:]
        self.cursor_pos += len(char)
        self.modified = True

    def delete_char(self, backward=True):
        """Delete character at cursor position"""
        line = self.lines[self.current_line]
        
        if backward and self.cursor_pos > 0:
            self.lines[self.current_line] = line[:self.cursor_pos-1] + line[self.cursor_pos:]
            self.cursor_pos -= 1
            self.modified = True
        elif not backward and self.cursor_pos < len(line):
            self.lines[self.current_line] = line[:self.cursor_pos] + line[self.cursor_pos+1:]
            self.modified = True

    def split_line(self):
        """Split line at cursor position"""
        line = self.lines[self.current_line]
        left_part = line[:self.cursor_pos]
        right_part = line[self.cursor_pos:]
        
        self.lines[self.current_line] = left_part
        self.lines.insert(self.current_line + 1, right_part)
        self.current_line += 1
        self.cursor_pos = 0
        self.modified = True

    def join_lines(self):
        """Join current line with next line"""
        if self.current_line < len(self.lines) - 1:
            current_len = len(self.lines[self.current_line])
            self.lines[self.current_line] += self.lines[self.current_line + 1]
            self.lines.pop(self.current_line + 1)
            self.cursor_pos = current_len
            self.modified = True

    def move_cursor(self, direction):
        """Move cursor in given direction"""
        if direction == 'left' and self.cursor_pos > 0:
            self.cursor_pos -= 1
        elif direction == 'right' and self.cursor_pos < len(self.lines[self.current_line]):
            self.cursor_pos += 1
        elif direction == 'up' and self.current_line > 0:
            self.current_line -= 1
            self.cursor_pos = min(self.cursor_pos, len(self.lines[self.current_line]))
        elif direction == 'down' and self.current_line < len(self.lines) - 1:
            self.current_line += 1
            self.cursor_pos = min(self.cursor_pos, len(self.lines[self.current_line]))
        elif direction == 'home':
            self.cursor_pos = 0
        elif direction == 'end':
            self.cursor_pos = len(self.lines[self.current_line])

    # MENU SYSTEM
    def show_menu(self):
        """Show the main menu"""
        self.mode = "MENU"
        self.menu_cursor = 0
        self.in_submenu = False
        self.submenu_items = []

    def show_settings_menu(self):
        """Show settings submenu"""
        self.in_submenu = True
        self.submenu_items = [
            f"Font Size: {self.current_font_size}",
            f"Line Numbers: {'ON' if self.show_linenumbers else 'OFF'}",
            f"Word Wrap: {'ON' if self.wrap_lines else 'OFF'}",
            f"Syntax Highlighting: {'ON' if self.syntax_highlighting else 'OFF'}",
            "Back to Main Menu"
        ]
        self.menu_cursor = 0

    def handle_menu_input(self, char):
        """Handle menu navigation and selection"""
        if char == '\x1b':  # ESC - exit menu
            self.mode = "EDIT"
            return
        
        elif char in ['j', '\x0e']:  # j or Ctrl+N - down
            max_items = len(self.submenu_items) if self.in_submenu else len(self.menu_items)
            self.menu_cursor = (self.menu_cursor + 1) % max_items
            
        elif char in ['k', '\x10']:  # k or Ctrl+P - up
            max_items = len(self.submenu_items) if self.in_submenu else len(self.menu_items)
            self.menu_cursor = (self.menu_cursor - 1) % max_items
            
        elif char == '\n' or char == '\r':  # ENTER - select
            self.execute_menu_action()
            
        elif char == 'i':  # i - immediate return to edit
            self.mode = "EDIT"

    def execute_menu_action(self):
        """Execute the selected menu action"""
        if self.in_submenu:
            self.execute_settings_action()
        else:
            self.execute_main_menu_action()

    def execute_main_menu_action(self):
        """Execute main menu action"""
        actions = {
            0: self.menu_save,           # Save
            1: self.menu_save_as,        # Save As
            2: self.menu_open_file,      # Open File
            3: self.menu_new_file,       # New File
            4: self.menu_settings,       # Settings
            5: self.menu_help,           # Help
            6: self.menu_quit           # Quit
        }
        
        action = actions.get(self.menu_cursor)
        if action:
            action()

    def execute_settings_action(self):
        """Execute settings menu action"""
        if self.menu_cursor == 0:  # Font Size
            self.change_font_size()
        elif self.menu_cursor == 1:  # Line Numbers
            self.show_linenumbers = not self.show_linenumbers
            self.save_config()
        elif self.menu_cursor == 2:  # Word Wrap
            self.wrap_lines = not self.wrap_lines
            self.save_config()
        elif self.menu_cursor == 3:  # Syntax Highlighting
            self.syntax_highlighting = not self.syntax_highlighting
        elif self.menu_cursor == 4:  # Back
            self.in_submenu = False

    def change_font_size(self):
        """Cycle through font sizes"""
        sizes = [12, 14, 16, 18, 20, 24, 30]
        current_index = sizes.index(self.current_font_size) if self.current_font_size in sizes else 2
        self.current_font_size = sizes[(current_index + 1) % len(sizes)]
        self.save_config()

    def menu_save(self):
        """Save current file"""
        if self.filename:
            if self.save_file():
                self.mode = "EDIT"
        else:
            self.menu_save_as()

    def menu_save_as(self):
        """Save as new file"""
        if self.save_file_as():
            self.mode = "EDIT"

    def menu_open_file(self):
        """Open file menu"""
        files = self.list_files()
        if files:
            # Simple file selection - in a real implementation you'd make a scrollable file list
            print("Available files:", ", ".join(files))
            filename = self.text_input("Open file: ")
            if filename and self.open_file(filename):
                self.mode = "EDIT"
        else:
            print("No text files found in current directory")

    def menu_new_file(self):
        """Create new file"""
        self.lines = [""]
        self.current_line = 0
        self.cursor_pos = 0
        self.filename = None
        self.modified = False
        self.mode = "EDIT"

    def menu_settings(self):
        """Show settings menu"""
        self.show_settings_menu()

    def menu_help(self):
        """Show help"""
        self.show_welcome()
        self.mode = "EDIT"

    def menu_quit(self):
        """Quit editor"""
        if self.modified:
            print("You have unsaved changes. Use Save from menu first.")
        else:
            self.cleanup()
            sys.exit(0)

    def text_input(self, prompt):
        """Simple text input for filenames"""
        print(f"\n{prompt}", end='', flush=True)
        # Temporarily restore normal terminal mode for input
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, self.old_settings)
        try:
            user_input = input()
        except KeyboardInterrupt:
            user_input = ""
        # Return to raw mode
        self.setup_terminal()
        return user_input

    # DISPLAY RENDERING
    def draw_display(self):
        """Redraw the entire display"""
        # Clear background
        self.draw.rectangle((0, 0, self.display.width, self.display.height), 
                          outline=self.bg_color, fill=self.bg_color)
        
        if self.mode == "MENU":
            self.draw_menu()
        else:
            self.draw_editor()
        
        # Update display
        self.display.image(self.image)
        self.display.show()

    def draw_editor(self):
        """Draw the text editor interface"""
        # Calculate dimensions
        line_height = self.current_font_size + 2
        status_height = 20 if self.show_status else 0
        line_num_width = 30 if self.show_linenumbers else 0
        
        # Draw line numbers area
        if self.show_linenumbers:
            self.draw.rectangle((0, 0, line_num_width, self.display.height - status_height),
                              outline=BLACK, fill=WHITE)
        
        # Draw text content
        y_pos = BORDER
        visible_lines = min(MAX_LINES, len(self.lines) - self.scroll_offset)
        
        for i in range(visible_lines):
            line_num = self.scroll_offset + i
            if line_num >= len(self.lines):
                break
            
            # Line number
            if self.show_linenumbers:
                self.draw.text((5, y_pos), f"{line_num + 1:3d}", 
                             font=self.get_font("mono", 12), fill=BLACK)
            
            # Text content
            line_text = self.lines[line_num]
            
            # Handle cursor on current line
            if line_num == self.current_line and self.mode == "EDIT":
                line_text = (line_text[:self.cursor_pos] + "|" + 
                           line_text[self.cursor_pos:])
            
            self.draw.text((line_num_width + BORDER, y_pos), line_text,
                         font=self.get_font("regular", self.current_font_size), 
                         fill=self.text_color)
            y_pos += line_height
            
            if y_pos > self.display.height - status_height - line_height:
                break
        
        # Draw status bar
        if self.show_status:
            self.draw_status_bar(status_height)

    def draw_menu(self):
        """Draw the menu interface"""
        # Draw menu background
        menu_width = 200
        menu_height = 180
        menu_x = (self.display.width - menu_width) // 2
        menu_y = (self.display.height - menu_height) // 2
        
        self.draw.rectangle((menu_x, menu_y, menu_x + menu_width, menu_y + menu_height),
                          outline=BLACK, fill=WHITE)
        
        # Draw menu title
        title = "SETTINGS" if self.in_submenu else "MAIN MENU"
        self.draw.text((self.display.width // 2 - 40, menu_y + 10), title,
                     font=self.get_font("bold", 18), fill=BLACK)
        
        # Draw menu items
        items = self.submenu_items if self.in_submenu else self.menu_items
        y_pos = menu_y + 40
        
        for i, item in enumerate(items):
            if i == self.menu_cursor:
                # Highlight selected item
                self.draw.rectangle((menu_x + 10, y_pos - 2, menu_x + menu_width - 10, y_pos + 18),
                                  outline=BLACK, fill=BLACK)
                text_color = WHITE
            else:
                text_color = BLACK
            
            self.draw.text((menu_x + 15, y_pos), item,
                         font=self.get_font("regular", 16), fill=text_color)
            y_pos += 25
        
        # Draw help text
        help_text = "↑↓: Navigate  ENTER: Select  ESC: Back  i: Edit"
        self.draw.text((menu_x + 10, menu_y + menu_height - 20), help_text,
                     font=self.get_font("mono", 12), fill=BLACK)

    def draw_status_bar(self, height):
        """Draw status bar at bottom"""
        status_bg = BLACK if self.bg_color == WHITE else WHITE
        status_text = WHITE if self.bg_color == WHITE else BLACK
        
        # Status bar background
        self.draw.rectangle((0, self.display.height - height, 
                           self.display.width, self.display.height),
                          outline=status_bg, fill=status_bg)
        
        # Status information
        mode_display = self.mode
        filename_display = self.filename or "[No Name]"
        if self.modified:
            filename_display += " *"
        
        status_info = f"{mode_display} | {filename_display} | Ln {self.current_line + 1}"
        self.draw.text((BORDER, self.display.height - height + 4),
                      status_info, font=self.get_font("mono", 12), fill=status_text)

    # MAIN EDITOR LOOP
    def run(self):
        """Main editor loop"""
        self.setup_terminal()
        self.draw_display()
        
        try:
            while True:
                char = self.get_char()
                if char:
                    self.handle_input(char)
                    self.draw_display()
        finally:
            self.cleanup()

    def handle_input(self, char):
        """Handle user input based on current mode"""
        if self.mode == "EDIT":
            self.handle_edit_input(char)
        elif self.mode == "MENU":
            self.handle_menu_input(char)

    def handle_edit_input(self, char):
        """Handle input in edit mode"""
        if char == '\x1b':  # ESC - show menu
            self.show_menu()
            
        elif char == '\n' or char == '\r':  # ENTER - new line (FIXED)
            self.split_line()
            
        elif char == '\x7f':  # Backspace
            if self.cursor_pos == 0 and self.current_line > 0:
                # Join with previous line
                self.current_line -= 1
                self.cursor_pos = len(self.lines[self.current_line])
                self.join_lines()
            else:
                self.delete_char(backward=True)
                
        elif char in ['\x02', '\x06', '\x0e', '\x10']:  # Navigation keys
            self.handle_navigation(char)
            
        elif char.isprintable():
            self.insert_char(char)

    def handle_navigation(self, char):
        """Handle navigation keys"""
        if char == '\x02':  # Ctrl+B - left
            self.move_cursor('left')
        elif char == '\x06':  # Ctrl+F - right
            self.move_cursor('right')
        elif char == '\x0e':  # Ctrl+N - down
            self.move_cursor('down')
        elif char == '\x10':  # Ctrl+P - up
            self.move_cursor('up')
        elif char == '\x01':  # Ctrl+A - home
            self.move_cursor('home')
        elif char == '\x05':  # Ctrl+E - end
            self.move_cursor('end')

def main():
    editor = AdvancedTextEditor()
    
    # Open file from command line if specified
    if len(sys.argv) > 1:
        editor.open_file(sys.argv[1])
    
    editor.run()

if __name__ == "__main__":
    main()