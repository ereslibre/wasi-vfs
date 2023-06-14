#include <stdio.h>

int main(int argc, char **argv) {
  FILE *demo = fopen("haiku.txt", "r");

  while (1) {
    int theChar = fgetc(demo);
    if (feof(demo))
      break;
    printf("%c", theChar);
  }

  return 0;
}
