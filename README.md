# pixelfield

A very simple application that allows manual input of "pixel fields", rectangular fields of black/white/unset pixels, e.g. 2D barcodes.

Uses an embarrassingly trivial JSON-based file format for save-and-restore purposes. Also contains built-in functionality to convert such files into PNG.

## UI

The top left corner contains a zoomed-in view of the part of the image currently being modified.

The top right corner contains the color currently being used to paint as well as the active direction.

The bottom right corner contains a view of the full image.

## Controls

* arrow keys: move the cursor through the image

* `0` through `9`: place that many pixels of the current color into the image, starting at the cursor and continuing in the active direction, then invert the current color

* `R` (reverse): switch the active direction

* `X` (exchange): invert the current color

* `S` (save): save the image

* `Home`: return to the top-left corner

* `T`: set the current pixel to true (black)

* `F`: set the current pixel to false (white)

* `Backspace` or `Delete`: clear the current pixel

By default, the active direction is rightward. Once the cursor hits the right edge, it is advanced to the next row and the active direction is inverted. This means that the default input strategy is boustrophedon (odd rows left-to-right, even rows right-to-left).
