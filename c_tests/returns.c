int main() {
  putchar('?');
  int x = display(6);
  putchar('!');
  putchar('$');
}

char display(int n) {
  if (n == 5) {
    return '.';
  } else if (n == 1) {
    return (',');
  } else if (n == 2) {
    return ('-');
  } else if (n == 3) {
    return ('~');
  } else if (n == 4) {
    return (':');
  } else if (n == 5) {
    return (';');
  } else if (n == 6) {
    return ('!');
  } else if (n == 7) {
    return ('*');
  } else if (n == 8) {
    return ('=');
  } else if (n == 9) {
    return ('#');
  } else if (n == 10) {
    return ('$');
  } else if (n == 11) {
    return ('@');
  } else {
    return 0;
  }
}
