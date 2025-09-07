int dz = 5;
int r1 = 1;
int r2 = 2;

int length_cordic(int x, int y, int *x2_, int y2) {
  int x2 = *x2_;
  if (x < 0) {
    x = -x;
    x2 = -x2;
  }
  for (int i = 0; i < 8; i++) {
    int t = x;
    int t2 = x2;
    if (y < 0) {
      x -= y >> i;
      y += t >> i;
      x2 -= y2 >> i;
      y2 += t2 >> i;
    } else {
      x += y >> i;
      y -= t >> i;
      x2 += y2 >> i;
      y2 -= t2 >> i;
    }
  }
  *x2_ = (x2 >> 1) + (x2 >> 3);
  return (x >> 1) + (x >> 3);
}

int main() {
  
}
