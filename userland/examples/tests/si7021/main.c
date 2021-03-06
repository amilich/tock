#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>

#include <tock.h>
#include <console.h>
#include <si7021.h>

int main () {
  putstr("[SI7021] Test App\n");

  // Start a measurement
  int humi, temp;
  si7021_get_temperature_humidity_sync(&temp, &humi);

  // Print the value
  printf("\tTemp(%d 1/100 degrees C) [0x%X]\n", temp, temp);
  printf("\tHumi(%d 0.01%%) [0x%X]\n\n", humi, humi);
}
