char r = 'A'-1;
char h = r + 4;
char i = r + 4*2;

int rec() {
  i++;
  putchar(i);
  rec();
}

int main() {
  h = h + 4;
  putchar(h);
  rec();
}
