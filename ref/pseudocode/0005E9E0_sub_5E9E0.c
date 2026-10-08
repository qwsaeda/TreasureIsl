/* ea:   0x0005E9E0 */
/* name: sub_5E9E0 */
/* size: 200 */
_DWORD *__fastcall sub_5E9E0(_DWORD *a1, int a2, _DWORD *a3, int *a4)
{
  int v7; // r0
  int v8; // r1
  int v9; // r2
  int v10; // r3
  int v11; // r1
  int v12; // r2
  int v13; // r3
  int v14; // r1

  v7 = *a4;
  v8 = a4[1];
  v9 = a4[2];
  v10 = a4[3];
  *a1 = v7;
  a1[1] = v8;
  a1[2] = v9;
  a1[3] = v10;
  v11 = a3[1];
  v12 = a3[2];
  v13 = a3[3];
  a1[4] = *a3;
  a1[5] = v11;
  a1[6] = v12;
  a1[7] = v13;
  sub_5EAA8(a1 + 4, a2);
  v14 = a1[6];
  *a1 += a1[4] - *a3;
  a1[2] += v14 - a3[2];
  a1[1] = a1[5] - a1[4] + *a1;
  a1[3] = a1[7] - a1[6] + a1[2];
  return a1;
}

