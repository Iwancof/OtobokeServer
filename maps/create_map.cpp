#include<stdio.h>
#include<stdlib.h>
#include<time.h>

#define swap(x, y) decltype(x) tmp = (x); (x) = (y); (y) = tmp;

void show(int field[30][30]);
void punch(int field[30][30], int sx, int sy, int ex, int ey);

int main() {
  srand((unsigned)time(NULL));

  int field[30][30];
  for(int x = 0;x < 30;x++) {
    for(int y = 0;y < 30;y++) {
      field[x][y] = 1;
    }
  }
  for(int i = 0;i < 10;i++) {
    int sx = rand() % 14;
    int ex = rand() % 14;
    int sy = rand() % 14;
    int ey = rand() % 14;
    sx *= 2; sx++;
    ex *= 2; ex++;
    sy *= 2; sy++;
    ey *= 2; ey++;

    if(ex < sx) {
      swap(sx, ex);
    }
    if(ey < sy) {
      swap(sy, ey);
    }

    punch(field, sx, sy, ex, ey); 
  }
  show(field);

}

void show(int field[30][30]) {
  printf("------------\n");
  for(int x = 0;x < 30;x++) {
    for(int y = 0;y < 30;y++) {
      printf("%d", field[x][y]);
    }
    printf("\n");
  }
  printf("------------\n");
}

void punch(int field[30][30], int sx, int sy, int ex, int ey) {
  for(int x = sx; x <= ex; x++)  {
    field[x][sy] = 5;
    field[x][ey] = 5;
  }
  for(int y = sy; y <= ey; y++)  {
    field[sx][y] = 5;
    field[ex][y] = 5;
  }
}


