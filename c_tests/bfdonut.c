int dz = 6, r1 = 1, r2 = 2;

int shift(int r, int n) {
  return r>>n;
}

#define R(s,x,y) x-=shift(y,s); y+=shift(x,s)

int length_cordic(int x, int y, int *x2_, int y2) {
  int x2 = *x2_;
  if (x < 0) { // start in right half-plane
    x = -x;
    x2 = -x2;
  }
  for (int i = 0; i < 8; i++) {
    int t = x;
    int t2 = x2;
    if (y < 0) {
      x -= shift(y , i);
      y += shift(t , i);
      x2 -= shift(y2 , i);
      y2 += shift(t2 , i);
    } else {
      x += shift(y , i);
      y -= shift(t , i);
      x2 += shift(y2 , i);
      y2 -= shift(t2 , i);
    }
  }
  // divide by 0.625 as a cheap approximation to the 0.607 scaling factor factor
  // introduced by this algorithm (see https://en.wikipedia.org/wiki/CORDIC)
  *x2_ = (shift(x2 , 1) + shift(x2 , 3));
  return (shift(x , 1) + shift(x , 3));
}

void main() {
  // high-precision rotation directions, sines and cosines and their products
  int sB = 0, cB = 16384;
  int sA = 11583, cA = 11583;
  int sAsB = 0, cAsB = 0;
  int sAcB = 11583, cAcB = 11583;

  for (;;) {
    int p0x = shift(dz * sB , 6);
    int p0y = shift(dz * sAcB , 6);
    int p0z = shift(-dz * cAcB , 6);

    int r1i = r1*256;
    int r2i = r2*256;

    int niters = 0;
    int nnormals = 0;
    int yincC = shift(cA , 6) + shift(cA , 5);      
    int yincS = shift(sA , 6) + shift(sA , 5);      
    int xincX = shift(cB , 7) + shift(cB , 6);      
    int xincY = shift(sAsB , 7) + shift(sAsB , 6);  
    int xincZ = shift(cAsB , 7) + shift(cAsB , 6);  
    int ycA = -(shift(cA , 1) + shift(cA , 4));     
    int ysA = -(shift(sA , 1) + shift(sA , 4));     
    for (int j = 0; j < 23; j++, ycA += yincC, ysA += yincS) {
      int xsAsB = shift(sAsB , 4) - sAsB;
      int xcAsB = shift(cAsB , 4) - cAsB; 

      int vxi14 = shift(cB , 4) - cB - sB;
      int vyi14 = ycA - xsAsB - sAcB;
      int vzi14 = ysA + xcAsB + cAcB;

      for (int i = 0; i < 79; i++, vxi14 += xincX, vyi14 -= xincY, vzi14 += xincZ) {
        int t = 512;

        int px = p0x + shift(vxi14 , 5);
        int py = p0y + shift(vyi14 , 5);
        int pz = p0z + shift(vzi14 , 5);
        int lx0 = shift(sB , 2);
        int ly0 = shift(sAcB - cA , 2);
        int lz0 = shift(-cAcB - sA , 2);
        for (;;) {
          int t0, t1, t2, d;
          int lx = lx0, ly = ly0, lz = lz0;
          t0 = length_cordic(px, py, &lx, ly);
          t1 = t0 - r2i;
          t2 = length_cordic(pz, t1, &lz, lx);
          d = t2 - r1i;
          t += d;

          if (t > 8*256) {
            putchar(' ');
            break;
          } else if (d < 2) {
            int N = shift(lz , 9);
            putchar(".,-~:;!*=#$@"[N > 0 ? N < 12 ? N : 11 : 0]);
            nnormals++;
            break;
          }

            px += shift(d*vxi14 , 14);
            py += shift(d*vyi14 , 14);
            pz += shift(d*vzi14 , 14);

          niters++;
        }
      }
      putchar(10);
    }

    R(5, cA, sA);
    R(5, cAsB, sAsB);
    R(5, cAcB, sAcB);
    R(6, cB, sB);
    R(6, cAcB, cAsB);
    R(6, sAcB, sAsB);

    
    putchar(27);
    putchar(91);
    putchar('2');
    putchar('3');
    putchar('A');
  }
}
