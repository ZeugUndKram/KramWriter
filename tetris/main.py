#!/usr/bin/env python3

import time
import board
import busio
import digitalio
import adafruit_sharpmemorydisplay
from collections import deque

# Import our modules
from tetris import Tetris
from display_manager import DisplayManager
from input_handler import InputHandler

def main():
    # Initialize the Sharp Memory Display
    spi = busio.SPI(board.SCK, MOSI=board.MOSI)
    scs = digitalio.DigitalInOut(board.D6)
    display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)
    
    print("\n" + "=" * 50)
    print("TETRIS - Optimized Version")
    print("Game board is centered on screen")
    print("Controls:")
    print("  Arrow keys: Move/Rotate/Fast drop")
    print("  Spacebar: Hard drop (instant)")
    print("  R: Restart game")
    print("  Q: Quit game")
    print("=" * 50)
    
    # Create game with display dimensions for proper centering
    game = Tetris(20, 10, display.width, display.height)
    game.new_figure()
    
    # Create display manager
    display_manager = DisplayManager(display, game)
    
    # Create and start input handler
    input_handler = InputHandler()
    input_handler.start()
    
    # Game timing
    last_drop_time = time.time()
    drop_interval = 0.5
    
    # Performance tracking
    frame_times = deque(maxlen=60)
    fps = 0
    input_lag = 0
    input_timestamps = deque(maxlen=30)
    
    # Game loop
    try:
        while True:
            frame_start = time.time()
            current_time = time.time()
            
            # 1. Process input (highest priority)
            input_keys = input_handler.get_keys()
            
            if input_keys:
                # Record input time for lag measurement
                input_timestamps.append(frame_start)
                
                for key in input_keys:
                    if key == ord(' '):  # Spacebar
                        if game.state == "start":
                            game.hard_drop()
                    elif key == ord('r') or key == ord('R'):
                        if game.state == "gameover" or game.state == "start":
                            game = Tetris(20, 10, display.width, display.height)
                            game.new_figure()
                            display_manager = DisplayManager(display, game)
                    elif key == ord('q') or key == ord('Q'):
                        print("\nQuitting game...")
                        return
                    elif game.state == "start":
                        if key == curses.KEY_UP:
                            game.rotate()
                        elif key == curses.KEY_DOWN:
                            game.go_down()
                        elif key == curses.KEY_LEFT:
                            game.go_side(-1)
                        elif key == curses.KEY_RIGHT:
                            game.go_side(1)
            
            # 2. Update game logic
            if game.state == "clearing":
                game.update_animation(current_time)
            
            # Auto-drop
            if game.state == "start" and current_time - last_drop_time > drop_interval:
                game.go_down()
                last_drop_time = current_time
                # Speed up gradually
                drop_interval = max(0.1, 0.5 - (game.score / 2000) * 0.1)
            
            # 3. Calculate performance metrics
            frame_end = time.time()
            frame_time = frame_end - frame_start
            frame_times.append(frame_time)
            
            # Calculate FPS
            if len(frame_times) > 0:
                avg_frame_time = sum(frame_times) / len(frame_times)
                fps = 1.0 / avg_frame_time if avg_frame_time > 0 else 0
            
            # Calculate input lag
            if input_timestamps:
                avg_time = sum(input_timestamps) / len(input_timestamps)
                input_lag = (current_time - avg_time) * 1000  # ms
            
            # 4. Update display (rate-limited by display_manager)
            display_updated = display_manager.update(fps, input_lag)
            
            # 5. Adaptive sleep to maintain ~120Hz game loop
            target_frame_time = 1.0 / 120.0  # 8.33ms per frame
            elapsed = time.time() - frame_start
            
            if elapsed < target_frame_time:
                sleep_time = target_frame_time - elapsed
                if sleep_time > 0.001:  # Only sleep if we have >1ms
                    time.sleep(sleep_time)
            
            # Print FPS every second for debugging
            if int(current_time) % 2 == 0 and int(frame_start) != int(current_time):
                print(f"FPS: {fps:.1f}, Lag: {input_lag:.0f}ms, State: {game.state}")
            
    except KeyboardInterrupt:
        print("\nGame interrupted")
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Clean up
        input_handler.stop()
        
        # Clear display
        display.fill(1)
        display.show()
        
        print("\nGame ended. Goodbye!")

if __name__ == "__main__":
    main()