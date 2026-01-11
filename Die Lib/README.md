# Raspberry Pi Memory Display library

Rust library to drive Sharp Memory Display modules
like [this one](https://www.adafruit.com/product/4694) from Adafruit.

It has built in line-diffing for faster screen updates. It will only
write changed lines to the SPI device, updating the screen
incrementally.
