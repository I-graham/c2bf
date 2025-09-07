char r = 'A'-1;
char h = r + 4;
char i = r + 4*2 + 1;
int main() {
  putchar(h + 4);
  putchar(i);
  h = h + 4;
  putchar(h);
  putchar(i);
}
