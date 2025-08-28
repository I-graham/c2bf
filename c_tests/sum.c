int a = 1;
int b = 2;
int c = a + b;

int f(int x, int y) {
  return x + y + 10;
}

int main() {
  int d = f(b, c) + 6;
  if (d <= 20) {
    print(0);
  }

  if (d >= 20) {
    print(1);
  } else {
    print(2);
  }
  print(d);
}
