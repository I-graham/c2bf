int fibs[10];

int main() {
  fibs[1] = 1;

  for (int i = 2; i < 10; i++) {
    fibs[i] = fibs[i - 1] + fibs[i - 2];
  }

  putchar(fibs[9]);
}
