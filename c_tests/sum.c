int a = 1;
int b = 2;
int c = a + b;

int f(int x, int y) {
  int z = x + y + 10;
  return z;
}

int main() {
  int d = f(b, c) + 6 << 2;
  if (d <= 20) {
    print(0);
  }

  if (d >= 20) {
    print(1);
  } else {
    print(2);
  }
  print(d);
  return 0;
}
