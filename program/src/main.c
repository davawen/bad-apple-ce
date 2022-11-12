////////////////////////////////////////
// { BAD APPLE } { 0.1.0 }
// Author: davawen
// License: MIT
// Description: d
////////////////////////////////////////

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <debug.h>
#include <graphx.h>
#include <fileioc.h>
#include <sys/timers.h>
#include <ti/getcsc.h>

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// #include "frames.h"

const uint24_t WIDTH = 320;
const uint8_t HEIGHT = 240;

uint24_t min(uint24_t a, uint24_t b) {
    return a < b ? a : b;
}
int main(void)
{
    timer_Enable(1, TIMER_32K, TIMER_NOINT, TIMER_UP);
	timer_Set(1, 0);

    gfx_Begin();
    gfx_SetDrawBuffer();

	uint8_t handle = 0;
	uint8_t handle_idx = 0;

    uint16_t idx = 0;
	uint16_t end = 0;

    for(;;) {
		if(idx >= end) {
			char buf[5];
			sprintf(buf, "f%d", handle_idx);
			// dbg_printf("loading var %s, index %u, got ", buf, handle_idx);
			ti_Close(handle);
			handle = ti_Open(buf, "r");
			// dbg_printf("%u\n", handle);
			if(handle == 0) goto end;

			handle_idx++;
			idx = 0;
			end = ti_GetSize(handle);
		}

        uint32_t target_timer = timer_Get(1) + 3277; // wait 100 ms (= 10 FPS)

        uint24_t x = 0;
        uint8_t y = 0;

        for(;;) {
            uint8_t color;
            unsigned int count;
            uint8_t p = ti_GetC(handle);
			idx++;

            if(p & 0b10000000) {
                uint8_t p2 = ti_GetC(handle);
				idx++;
                color = (p2 & 0x1) * 255;
                count = (((unsigned int)p & 0b01111111) << 7) | ((unsigned int)p2 >> 1);
				count *= 2;
            }
            else {
                color = (p & 0x1) * 255;
                count = (p & (~0x1)) /*>> 1*/ ;
                // Don't shift to the right, since scaling needs it to be multiplied by 2 anyway
            }

            gfx_SetColor(color);
            while(count > 0) {
                uint8_t length = min(count, 240 - y);
                gfx_VertLine_NoClip(x, y, length);
                gfx_VertLine_NoClip(x + 1, y, length);

                count -= length;
                y += length;
                if(y == HEIGHT) {
                    y = 0;
                    x += 2;
					if(x == WIDTH) goto frame_end;
                }
            }
        }
	frame_end:

        gfx_SwapDraw();

		while(timer_Get(1) < target_timer) {}
    }
	
	gfx_SetColor(0);

    /* Wait for a key press */
    while (!os_GetCSC());

end:

    gfx_End();
    // timer_Disable(1);

    return 0;
}
