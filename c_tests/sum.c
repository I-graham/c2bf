int a = 1;
int b = 2;
int c = a + b;

int f(int x, int y) {
  int z = x + y + 16;
  return z;
}

int main() {
  int d = f(b, c) + 6 << 1;
  putchar(d);
  if (d <= 20) {
    putchar('0');
  }

  if (d >= 20) {
    putchar('1');
  } else {
    putchar('2');
  }
  putchar(d);
  return 0;
}
