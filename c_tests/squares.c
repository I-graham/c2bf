int is_square(int n) {
  int out = 0;
  for(int i = 1; i * i <= n;) {
    if (i*i == n) out = 1;
    i = i + 1;
  }
  return out;
}

int main() {
  for (int i = 1; i <= 100;) {
    if (is_square(i)) print(i);
    i = i + 1;
  }
}
