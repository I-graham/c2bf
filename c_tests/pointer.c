int *v;

int inc() {
  *v = 5;
  return 1;
}

int main() {
  int x = 7;

  putchar('0'+x);
  v = &x;
  inc();
  putchar('0'+x);
}
