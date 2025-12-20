#include "display_client.h"
#include <stdio.h>
#include <unistd.h>

int main() {
    printf("Connecting to display server...\n");
    
    if (display_init() < 0) {
        fprintf(stderr, "Failed to connect. Is display_server.py running?\n");
        return 1;
    }
    
    printf("Connected!\n");
    
    // Test 1: Clear display
    printf("Clearing display...\n");
    display_clear();
    sleep(1);
    
    // Test 2: Show logo (zentriert mit x=-1, y=-1)
    printf("Showing logo...\n");
    display_image("assets/logo.bmp", -1, -1);
    sleep(2);
    
    // Test 3: Text
    printf("Showing text...\n");
    display_clear();
    display_text(10, 50, 30, "Hello from C!");
    display_text(10, 100, 20, "This is FAST!");
    sleep(2);
    
    // Test 4: Rectangles
    printf("Drawing rectangles...\n");
    display_clear();
    display_rect(50, 50, 100, 80, false);  // Outline
    display_rect(200, 50, 100, 80, true);  // Filled
    sleep(2);
    
    // Test 5: Raw buffer with graphics
    printf("Drawing with raw buffer...\n");
    uint8_t *buffer = display_create_buffer();
    
    // Draw border
    display_draw_rect_buffer(buffer, 0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT, true, false);
    
    // Draw some lines
    display_draw_line_buffer(buffer, 0, 0, 399, 239, true);
    display_draw_line_buffer(buffer, 399, 0, 0, 239, true);
    display_draw_line_buffer(buffer, 200, 0, 200, 239, true);
    display_draw_line_buffer(buffer, 0, 120, 399, 120, true);
    
    // Draw some rectangles
    display_draw_rect_buffer(buffer, 50, 50, 80, 60, true, true);
    display_draw_rect_buffer(buffer, 270, 130, 80, 60, true, false);
    
    // Send to display
    display_raw_buffer(buffer, DISPLAY_BUFFER_SIZE);
    display_free_buffer(buffer);
    
    printf("Demo complete! Press Enter to exit...\n");
    getchar();
    
    display_close();
    return 0;
}