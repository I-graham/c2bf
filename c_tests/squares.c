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
    print(is_square(i)+'0');
    i = i + 1;
  }
}
