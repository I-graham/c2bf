int z[10];

int main() {
  z[1] = 1;

  for (int i = 2; i < 10; i++) {
    z[i] = z[i - 1] + z[i - 2];
  }

  putchar('0' + z[0]);
  putchar('0' + z[1]);
  putchar(z[9]);
}
