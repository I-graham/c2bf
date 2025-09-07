char r = 'A'-1;
char h = r + 4;
char i = r + 4*2;

int inner() {
  i++;
  putchar(i);
}

int main() {
  h = h + 4;
  putchar(h);
  inner();
}
