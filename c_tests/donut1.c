char display(unsigned short n);

unsigned short buff[1760];
unsigned short z[1760];

unsigned short shl(unsigned short x, unsigned short s) {
  return x<<s;
}

unsigned short shr(unsigned short x, unsigned short s) {
  short b = x & (1<<15);

  for (short v = 0; v < s; v++) {
    x = x >> 1;
    x = x | b;
  }
  
  return x;
}

unsigned short div(unsigned short p, unsigned short q) {
  short b = (1<<15)&p;
  if (b) {
    p = -p;
    return -(p / q);
  }
  return p / q;
}

unsigned short to_char(unsigned short p) {
  unsigned short b = (1<<15) & p;
  if (b) {
    p = -p;
    return -(p & 255);
  }
  return p & 255;
}

unsigned short lt(unsigned short lhs, unsigned short rhs) {
  unsigned short lb = (1<<15) & lhs;
  unsigned short rb = (1<<15) & rhs;

  if (lb && !rb) {
    return 1;
  } 
  if (rb && !lb) {
    return 0;
  }

  if (lb && rb) {
    lhs = -lhs;
    rhs = -lhs;
    return rhs < lhs;
  }

  return lhs < rhs;  
}

unsigned short main() {
  unsigned short i, j, f;
  unsigned short precision = 6;
  unsigned short c_3 = shl(3,2*precision);
  unsigned short sA=shl(1,precision), cA = 0, sB=shl(1,precision), cB=0;
  while (1) {
    for (i = 0; i < 1760; i++) {
      buff[i] = 32;
      z[i] = 127;
    }

    unsigned short sj=0, cj=shl(1,precision);
    for (j = 0; j < 90; j++) {
      unsigned short si=0, ci=shl(1,precision);
      for (i = 0; i<324; i++) {
        unsigned short R1=1, R2=shl(2,precision), K2=shl(5, 2*precision);
        unsigned short x0 = R1*cj + R2;
        unsigned short x1 = shr(ci*x0,precision);
        unsigned short x2 = shr(cA*sj, precision);
        unsigned short x3 = shr(si*x0, precision);
        unsigned short x4 = R1*x2 - shr(sA*x3,precision);
        unsigned short x5 = shr(sA*sj, precision);
        unsigned short im = shl(R1*x5,precision) + cA*x3;
        unsigned short x6 = K2 + im;
        unsigned short x7 = shr(cj*si,precision);
        x6 = div(x6, 15);
        unsigned short px = div(2* (cB*x1 - sB*x4), x6);
        unsigned short x = 40 + px;
        unsigned short y = 12 + div(  (cB*x4 + sB*x1),x6);
        unsigned short N = shr(shr(-cA*x7 - cB*(shr(-sA*x7,precision) + x2) - ci*shr(cj*sB,precision),precision) - x5, precision-3);

        unsigned short o = x + 80 * y;
        unsigned short zz = to_char(shr(im, precision+5));
        
        if (0 < y && y < 22 && 0 < x && x < 80 && lt(zz, z[o])) {
          z[o] = zz;
          buff[o] = display(N);
        }
        f = ci;
        ci -= shr(5*si,precision-2);
        si += shr(5*f, precision-2);
        f = shr(c_3-ci*ci-si*si, precision+1);
        ci = shr(ci*f, precision);
        si = shr(si*f, precision);
      }
      f = cj;
      cj -= shr(9*sj,precision-3);
      sj += shr(9*f, precision-3);
      f = shr(c_3-cj*cj-sj*sj, precision+1);
      cj = shr(cj*f,precision);
      sj = shr(sj*f,precision);
    }

    for (unsigned short k = 0; k < 1761; k++) {
      putchar(k % 80 ? buff[k] : 10);
    }

    f = cA;
    cA -= shr(5*sA,(precision-3));
    sA += shr(5*f,(precision-3));
    f = shr(c_3-cA*cA-sA*sA,(precision+1));
    cA = shr(cA*f,precision);
    sA = shr(sA*f,precision);

    f = cB;
    cB -= shr(5*sB,(precision-2));
    sB += shr(5*f,(precision-2));
    f = shr(c_3-cB*cB-sB*sB,(precision+1));
    cB = shr(cB*f,precision);
    sB = shr(sB*f,precision);
    

    putchar(27);
    putchar(91);
    putchar('2');
    putchar('3');
    putchar('A');
  }
}

char display(unsigned short n) {
  short b = n & (1<<15);
  if (b || n == 0) {
    return ('.');
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
