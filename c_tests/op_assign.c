int x;

int* pointer() {
  putchar('?');
  return &x;
}

int main() {
  x = '2';
  putchar(x);

  *pointer() += 2;

  putchar(x);
}
